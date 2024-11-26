use binrw::{binrw, helpers::until_eof};
use flate2::read::ZlibDecoder;
use std::io::SeekFrom;

mod checksum;
mod error;

pub use crate::{
    checksum::{FirmwareChecksum, SECRET_VALUE},
    error::Error,
};
pub type Result<T> = std::result::Result<T, Error>;

/// ATEM firmware image file
///
/// ## File format
///
/// * `0x20` bytes: firmware image checksum
/// * repeated until eof: [Resource]
///
/// ## Checksum
///
/// The image checksum is SHA-256 of the rest of the file, but has been
/// initialised with a "secret value":
/// `e6141730dd0a0c465941255d110f030545504239`.
///
/// ## Firmware location
///
/// Firmware files named `data-[0-9a-f]{4}.bin`.
///
/// They can be found in the `AdminUtility/PlugIns/{product_name}/Resources/`
/// subfolder of the application's install path:
///
/// * macOS: `/Library/Application Supports/Blackmagic Design/*/`
/// * Windows: `Program Files/Blackmagic Design/*/`
#[binrw]
#[derive(Debug, Default, Clone)]
#[brw(big, stream = r, map_stream = FirmwareChecksum::new)]
pub struct FirmwareFile {
    #[brw(pad_before = 32)]
    #[br(parse_with = until_eof)]
    pub resources: Vec<Resource>,

    #[brw(seek_before = SeekFrom::Start(0))]
    #[br(temp, assert(checksum == r.check(), "bad checksum: {:x?} != {:x?}", checksum, r.check()))]
    #[bw(calc(r.check()))]
    pub checksum: [u8; 0x20],
}

/// Firmware resource
///
/// ## Data format
///
/// Resource headers are at minimum 24 bytes:
///
/// * `u16`: magic value: `0xBDBD`
/// * `u16`: format version? always 1
/// * `u32`: ?
/// * `u32`: total resource size in bytes, including headers
/// * `u16`: header length
/// * `bool`: compression enabled (zlib)
/// * `u8`: type
/// * `u32`: unpacked length
/// * `u32`: maybe CRC?
/// * `header_length - 24` bytes: additional headers
///
/// The payload (`resource_size - header_length`) follows.
#[binrw]
#[derive(Default, PartialEq, Clone)]
#[br(assert(usize::from(header_length) >= Self::MIN_LENGTH))]
#[brw(big, magic = 0xBDBD0001u32)]
pub struct Resource {
    unknown1: u32,
    #[bw(try_calc(u32::try_from(payload.len() + header.len() + Self::MIN_LENGTH)))]
    length: u32,
    #[bw(try_calc(u16::try_from(header.len() + Self::MIN_LENGTH)))]
    header_length: u16,

    /// Compression enabled; 1 = zlib
    pub compression: u8,
    pub typ: u8,
    unpacked_length: u32,
    unknown7: u32,

    #[br(count = header_length - 24)]
    pub header: Vec<u8>,

    #[br(count = length - u32::from(header_length))]
    pub payload: Vec<u8>,
}

impl Resource {
    pub const MIN_LENGTH: usize = 24;

    pub fn decompress_payload(&self) -> ZlibDecoder<&[u8]> {
        ZlibDecoder::new(self.payload.as_ref())
    }
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("unknown1", &self.unknown1)
            .field("compression", &self.compression)
            .field("typ", &self.typ)
            .field("unpacked_length", &self.unpacked_length)
            .field("unknown7", &self.unknown7)
            .field("header", &hex::encode(&self.header))
            .field("payload", &format!("{} bytes", self.payload.len()))
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

    #[test]
    fn firmware_atem_mini() -> Result<()> {
        let mut cmd = hex::decode("BDBD000100000000007328300030000000732800F0B8512A0001001800000000000000011EDBBE490000000000000000")?;
        cmd.resize(cmd.len() + 7546880, 0);

        let expected = Resource {
            unknown1: 0,
            compression: 0,
            typ: 0,
            unpacked_length: 7546880,
            unknown7: 0xF0B8512A,
            header: hex::decode("0001001800000000000000011EDBBE490000000000000000")?,
            payload: vec![0u8; 7546880],
        };
        let r = Resource::read(&mut Cursor::new(&cmd))?;
        assert_eq!(expected, r);

        let mut out = Cursor::new(Vec::with_capacity(cmd.len() + 32));
        let fw = FirmwareFile {
            resources: vec![expected],
        };
        fw.write(&mut out)?;

        let out = out.into_inner();
        // Only compare the first 512 bytes, otherwise errors are hard to read
        assert_eq!(&cmd[..512], &out[32..544]);
        assert_eq!(
            hex::decode("05a5716ebe99b9fbbc06b0e9add2f12cc8362f2541ba7ee19b8f5593375ab209")?,
            &out[..32],
        );

        // Reading back the firmware should work with that checksum
        let _ = FirmwareFile::read(&mut Cursor::new(&out))?;
        Ok(())
    }

    #[test]
    fn firmware_camera_control_panel() -> Result<()> {
        let mut cmd = hex::decode("BDBD00010000000000002F4600300100000058D4337D01C40001001800000000000300011EDBBE0E0000000000000000")?;
        cmd.resize(cmd.len() + 12054, 0);

        let expected = Resource {
            unknown1: 0,
            compression: 1,
            typ: 0,
            unpacked_length: 22740,
            unknown7: 0x337D01C4,
            header: hex::decode("0001001800000000000300011EDBBE0E0000000000000000")?,
            payload: vec![0u8; 12054],
        };

        let r = Resource::read(&mut Cursor::new(&cmd))?;
        assert_eq!(expected, r);

        let mut out = Cursor::new(Vec::with_capacity(cmd.len() + 32));
        let fw = FirmwareFile {
            resources: vec![expected],
        };
        fw.write(&mut out)?;

        let out = out.into_inner();
        // Only compare the first 512 bytes, otherwise errors are hard to read
        assert_eq!(&cmd[..512], &out[32..544]);
        assert_eq!(
            hex::decode("76d7bb1d94be3022d096a844236797e35575f111b0cd1efe3a8ee2d06096cf60")?,
            &out[..32],
        );

        // Reading back the firmware should work with that checksum
        let _ = FirmwareFile::read(&mut Cursor::new(&out))?;

        let mut cmd = hex::decode("BDBD00010000000000006D840030010400040000F2C9BEE80001001800000000FF0000011EDBBE0E0000000000000000")?;
        cmd.resize(cmd.len() + 27988, 0);

        let expected = Resource {
            unknown1: 0,
            compression: 1,
            typ: 4,
            unpacked_length: 262144,
            unknown7: 0xf2c9bee8,
            header: hex::decode("0001001800000000FF0000011EDBBE0E0000000000000000")?,
            payload: vec![0u8; 27988],
        };
        let r = Resource::read(&mut Cursor::new(&cmd))?;
        assert_eq!(expected, r);

        let mut out = Cursor::new(Vec::with_capacity(cmd.len() + 32));
        let fw = FirmwareFile {
            resources: vec![expected],
        };
        fw.write(&mut out)?;

        let out = out.into_inner();
        // Only compare the first 512 bytes, otherwise errors are hard to read
        assert_eq!(&cmd[..512], &out[32..544]);
        assert_eq!(
            hex::decode("af21c869f1396f1fcb24b382002ac800b00e0d2986e6e80bd899f78d1c5e6078")?,
            &out[..32],
        );

        // Reading back the firmware should work with that checksum
        let _ = FirmwareFile::read(&mut Cursor::new(&out))?;
        Ok(())
    }
}
