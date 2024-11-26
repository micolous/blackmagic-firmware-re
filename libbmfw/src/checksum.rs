use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom, Write};

/// "Secret value" used to seed the firmware SHA-256 checksum.
pub const SECRET_VALUE: [u8; 20] = [
    0xe6, 0x14, 0x17, 0x30, 0xdd, 0x0a, 0x0c, 0x46, 0x59, 0x41, 0x25, 0x5d, 0x11, 0x0f, 0x03, 0x05,
    0x45, 0x50, 0x42, 0x39,
];

/// Firmware checksummer wrapper for `binrw`.
///
/// This assumes that binrw will always read or write the file sequentially, as
/// the checksum is at the start of the file.
pub struct FirmwareChecksum<T> {
    inner: T,
    sha256: Sha256,
    p: u64,
}

impl<T> FirmwareChecksum<T> {
    pub fn new(inner: T) -> Self {
        let mut sha256 = Sha256::new();
        sha256.update(SECRET_VALUE);
        Self {
            inner,
            sha256,
            p: 0,
        }
    }

    pub fn check(&self) -> [u8; 32] {
        self.sha256.clone().finalize().into()
    }
}

impl<T: Read> Read for FirmwareChecksum<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.inner.read(buf)?;
        if self.p < 32 {
            // Don't hash the first 32 bytes of the file.
            // Position of byte 32 in this buffer; self.p < 32 so this is safe to cast
            let x = (32 - self.p) as usize;

            if x < size {
                // Byte 32 is in this buffer
                self.sha256.update(&buf[x..size]);
            }
        } else {
            // We're past the first 32 bytes of the file.
            self.sha256.update(&buf[0..size]);
        }
        self.p += size as u64;
        Ok(size)
    }
}

impl<T: Seek> Seek for FirmwareChecksum<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.p = self.inner.seek(pos)?;
        Ok(self.p)
    }
}

impl<T: Write> Write for FirmwareChecksum<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self.inner.write(buf)?;
        if self.p < 32 {
            // Don't hash the first 32 bytes of the file
            // Position of byte 32 in this buffer; self.p < 32 so this is safe to cast
            let x = (32 - self.p) as usize;

            if x < size {
                // Byte 32 is in this buffer
                self.sha256.update(&buf[x..size]);
            }
        } else {
            // We're past the first 32 bytes of the file.
            self.sha256.update(&buf[..size]);
        }
        self.p += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
