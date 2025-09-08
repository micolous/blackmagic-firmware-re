# Blackmagic firmware reverse engineering tools

This repository contains tools aid reverse engineering Blackmagic Design
(ATEM, Web Presenter, etc.) devices' firmware/gateware update packages.

There are two parts to this software:

* `libbmfw`: library for parsing update packages
* `bmfw`: CLI tool for inspecting and extracting update packages

As this software is for reverse engineering undocumented file formats and has a
pre-1.0 version label, its APIs, interfaces and MSRV should be considered
*unstable* and *subject to change at any time*.

This is **not** intended to be a complete "solution" for reverse engineering or
reflashing these devices. Other tools like [Ghidra][] (with [plugins][mb-be])
may help you here.

[Ghidra]: https://ghidra-sre.org/
[mb-be]: https://github.com/embogit/Ghidra-MicroBlaze/pull/1

## Getting started

There are no official binary builds of this software, so you'll need to
[install a *recent* Rust compiler toolchain with `rustup`][install-rust].

[install-rust]: https://www.rust-lang.org/tools/install

You can then build everything with:

```sh
cargo build --release
```

The binary will be in `target/release/bmfw` or `target/release/bmfw.exe`.

You can then extract an update package with:

```sh
./target/release/bmfw extract /path/to/data-bde6.bin -o /tmp/bde6
```

## Finding update packages

Update packages are named `data-${PID}.bin`, matching
[the device's USB product ID](#blackmagic-device-list) in lower case.

Some devices like the ATEM Television Studio have extra files for UI
translations, which are suffixed with a language code.

These packages can be found in a few places:

* macOS:
  * `/Applications/Blackmagic ATEM Switchers/ATEM Setup.app/Contents/Resources`
  * `/Library/Application Supports/Blackmagic Design/*/AdminUtility/Plugins/*/Resources`
* Windows:
  * `Program Files/Blackmagic Design/*/Setup/AdminUtility/Plugins/*/Resources`
  * `Program Files (x86)/Blackmagic Design/*/Atem Setup/PlugIns/*/Resources`

You can find which firmware versions are included with a Blackmagic setup tool
via its debug window: hold the <kbd>Shift</kbd> key and double-click the lower
left corner of the window where you have a normal mouse cursor (rather than a
resize handle).

The debug window *cannot* be accessed by pressing <kbd>Tab</kbd> or using
accessibility tools.

## Extract update packages from installers *without* installing

Blackmagic's software only allows one version to be installed at a time. This
can be troublesome when comparing versions.

It is possible to extract update packages from the
[macOS](#extract-macos-installers) and [Windows](#extract-windows-installers)
installers *without* installing the software.

### Extract macOS installers

Blackmagic software on macOS is distributed as a ZIP file containing a
`.dmg` file containing one or more [Installer `.pkg` files][macos-installer].
`.pkg` files are [XAR files][xar], which can be extracted with the 7-Zip CLI
(`7z`).

[macos-installer]: https://en.wikipedia.org/wiki/List_of_built-in_macOS_apps#Installer
[xar]: https://en.wikipedia.org/wiki/Xar_(archiver)

First, find the `Payload` segment of the main part of the `.pkg` file:

```sh
7z l "/Volumes/Blackmagic .../Install ....pkg" | grep Payload
```

For example, the main part of the ATEM Switchers package is in
`Switchers.pkg/Payload`.

Next, extract that part:

```sh
mkdir /tmp/extract
cd /tmp/extract
7z e "/Volumes/Blackmagic .../Install ....pkg" PackageNameHere.pkg/Payload
```

`Payload` is a `cpio` file, which can also be inspected and extracted with `7z`.
To find files named like update packages (`data-[0-9a-f]{4}(-[^/]+)?\.bin`):

```sh
7z l Payload | egrep "data-[0-9a-f]{4}(-[^/]+)?\\.bin"
```

Entries will have the full path on disk where it'd be normally installed.

Once you've found the update package(s) you're interested in, extract with:

```sh
7z e Payload "./Applications/....app/Contents/Resources/data-ABCD.bin
```

### Extract Windows installers

Blackmagic software on Windows is distributed as a ZIP file containing an `.msi`
file, and sometimes also with one or more *hidden* [`.cab` files][cab]. All
files can be extracted with the 7-Zip GUI or its CLI (`7z`).

[cab]: https://en.wikipedia.org/wiki/Cabinet_(file_format)

You can check if a `.cab` contains update packages by looking for files named
like update packages with a shell script:

```sh
7z l ExampleSupportFile.cab | egrep "data_[0-9a-f]{4}(_[^/]+)?\\.bin"
```

For `.msi` files, the file names are a litle bit different:

```sh
7z l ExampleInstaller.msi | egrep -i "Data[0-9a-f]{4}"
```

You can extract these using similar steps to the
[macOS instructions](#extract-macos-installers) (above).

Doing this with PowerShell is left as an exercise for the reader (as the Windows
COM APIs for inspecting archives are not easy to use). Alternatively, you could
run those shell scripts in [MSYS2][] or [WSL][]. 😉

[MSYS2]: https://www.msys2.org/
[WSL]: https://learn.microsoft.com/en-us/windows/wsl/about

## Data structure descriptions

There are notes for each data structure available as
[`rustdoc` comments](https://doc.rust-lang.org/rustdoc/). You can access these
via:

* [docs.rs of the latest tagged release](https://docs.rs/libbmfw/latest/libbmfw/)
* Build the current docs locally with `cargo doc -p libbmfw`
* Read source file comments starting with `///` in `./libbmfw/src`

## Update package structure

Blackmagic update package files consist of one or more resources:

* FPGA gateware, which is used for SDI/HDMI capture/switching and runs a
  MicroBlaze or PowerPC softcore
* ELF with software for the MicroBlaze (32-bit BE or LE) or PowerPC core
* `tar` archive containing non-executable resources (eg: fonts, images, strings)
* `tar` archive containing (encrypted?) gateware / software

## Blackmagic device list

> [!IMPORTANT]
> This is a list of known Blackmagic product IDs - **not** a list of "supported"
> devices.

All devices have USB VID `0x1edb` (Blackmagic Design).

"Family" refers to the setup / admin utility plugin used to configure and/or
update the device.

USB PID  | Family | Device
-------- | ------ | -------
**`0xbd__`** | ... | ... 
`0xbd15` | Converters | [MiniSdiToAnalog][mini-converter]
`0xbd16` | Converters | [MiniAnalogToSdi][mini-converter]
`0xbd17` | Converters | MiniSdiToHdmi
`0xbd18` | Converters | MiniHdmiToSdi
`0xbd19` | Converters | [MiniSyncGenerator][mini-converter]
`0xbd27` | Converters | [MiniAudioToSdi][mini-converter]
`0xbd28` | Converters | [MiniSdiToAudio][mini-converter]
`0xbd2c` | Converters | DVIExtender
`0xbd2e` | Converters | OgSdiToAnalog
`0xbd2f` | Converters | OgAnalogToSdi
`0xbd30` | Converters | OgSdiToHdmi
`0xbd31` | Converters | OgHdmiToSdi
`0xbd32` | Converters | OgSyncGenerator
`0xbd33` | Converters | OgAudioToSdi
`0xbd34` | Converters | OgSdiToAudio
`0xbd41` | Converters | [MiniUpDownCross][mini-converter]
`0xbd46` | Converters | [MiniAnalogToSdi2][mini-converter]
`0xbd47` | Converters | [MiniHdmiToSdi2][mini-converter]
`0xbd4a` | ATEM | ATEM 1 M/E Broadcast Panel
`0xbd50` | ATEM | GPI and Tally Interface
`0xbd53` | Converters | OgUpDownCross
`0xbd54` | Converters | AtemCameraConverter
`0xbd55` | Converters | AtemStudioConverter
`0xbd57` | ATEM | ATEM 2 M/E Broadcast Panel
`0xbd5f` | Converters | BatteryHdmiToSdi
`0xbd60` | Converters | BatterySdiToHdmi
`0xbd69` | Converters | [MiniSdiToHdmi4K][mini-converter]
`0xbd6a` | Converters | MiniSdiMux4K
`0xbd82` | Converters | AtemStudioConverter2
`0xbd83` | Converters | MiniSdiMux4K2
`0xbd84` | Converters | [MiniSdiToHdmi4K2][mini-converter]
`0xbd85` | Converters | [MiniOpticalFiber4K][mini-converter]
`0xbd91` | Converters | [MiniSdiToAnalog4K][mini-converter]
`0xbd92` | Converters | [MiniSdiDistribution4K][mini-converter]
`0xbd93` | Converters | [MiniSdiToAudio4K][mini-converter]
`0xbd94` | Converters | [MiniAudioToSdi4K][mini-converter]
`0xbd95` | Converters | [MiniSdiToHdmi4K3][mini-converter]
`0xbd96` | Converters | [MiniHdmiToSdi4K][mini-converter]
`0xbdb8` | Converters | Talkback8
`0xbdc5` | Converters | MicroSdiToHdmi
`0xbdc6` | Converters | MicroHdmiToSdi
`0xbdcf` | Converters | OgAnalogToSdiV2
`0xbdd0` | Converters | OgHdmiToSdiV2
`0xbde5` | Web Presenter | Web Presenter (rear USB Type-B / UVC out)
`0xbde6` | Web Presenter | Web Presenter (front USB Mini-B / ATMEL BMDUSB01)
`0xbde7` | Converters | MiniAudioToSdi2
`0xbdf2` | Converters | MiniSdiToHdmi4K4
`0xbdf6` | Converters | MiniHdmiToSdi4K2
**`0xbe__`** | ... | ... 
`0xbe00` | Converters | [OpticalFiber12G][mini-converter]
`0xbe06` | Converters | MiniUpDownCrossHD
`0xbe0c` | Converters | MicroBiDirectional
`0xbe0e` | ATEM | [ATEM Camera Control Panel][atem-cam]
`0xbe12` | Web Presenter | Web Presenter (according to debug window; update package is `data-bde6.bin`)
`0xbe25` | ATEM | ATEM Television Studio HD
`0xbe26` | ATEM | ATEM Television Studio Pro HD
`0xbe2c` | ATEM | ATEM Television Studio Pro 4K
`0xbe33` | ATEM D2 WebView | [ATEM Constellation 8K][atem-8k]
`0xbe48` | ATEM | ATEM 1 M/E Production Switcher
`0xbe49` | ATEM D2 | ATEM Mini
`0xbe4a` | ATEM | [ATEM 1 M/E Advanced Panel 10][adv-panel]
`0xbe4b` | ATEM | ATEM 4 M/E Broadcast Studio 4K
`0xbe52` | ATEM | ATEM Television Studio
`0xbe55` | ATEM D2 WebView | [ATEM Mini Pro][atem-mini]
`0xbe55` | ATEM D2 WebView | [ATEM Mini Pro ISO][atem-mini]
`0xbe57` | ATEM D2 | [ATEM 2 M/E Advanced Panel 20][adv-panel]
`0xbe58` | ATEM D2 | [ATEM 4 M/E Advanced Panel 40][adv-panel]
`0xbe5c` | ATEM | ATEM 2 M/E Production Switcher
`0xbe6e` | ATEM | ATEM Production Studio 4K
`0xbe6f` | Micro Converters | [Micro Converter SDI to HDMI 3G (v9)][micro-converter]
`0xbe70` | Micro Converters | [Micro Converter HDMI to SDI 3G (v9)][micro-converter]
`0xbe73` | Web Presenter HD | [Web Presenter HD][web-presenter]
`0xbe74` | Streaming Bridge | ATEM Streaming Bridge
`0xbe77` | Micro Converters | [Micro Converter SDI to HDMI 12G][micro-converter]
`0xbe78` | Micro Converters | [Micro Converter HDMI to SDI 12G][micro-converter]
`0xbe79` | Micro Converters | [Micro Converter BiDirectional SDI/HDMI 3G][micro-converter]
`0xbe7c` | ATEM D2 WebView | [ATEM Mini Extreme][atem-mini]
`0xbe7e` | ATEM D2 WebView | [ATEM 1 M/E Constellation HD][atem-constellation]
`0xbe7f` | ATEM D2 WebView | [ATEM 2 M/E Constellation HD][atem-constellation]
`0xbe81` | ATEM | ATEM 1 M/E Production Studio 4K
`0xbe83` | ATEM D2 WebView | [ATEM Mini Extreme ISO][atem-mini]
`0xbe86` | ATEM D2 WebView | [ATEM 4 M/E Constellation HD][atem-constellation]
`0xbe87` | ATEM | ATEM 2 M/E Production Studio 4K
`0xbe89` | Micro Converters | [Micro Converter BiDirectional SDI/HDMI 12G][micro-converter]
`0xbe8b` | Web Presenter HD | [Web Presenter 4K][web-presenter]
`0xbe90` | Micro Converters | [Micro Converter SDI to HDMI 3G (v10)][micro-converter]
`0xbe91` | Micro Converters | [Micro Converter HDMI to SDI 3G (v10)][micro-converter]
`0xbe95` | ATEM D2 | [ATEM SDI][atem-sdi]
`0xbe97` | ATEM D2 WebView | [ATEM SDI Pro ISO][atem-sdi]
`0xbe99` | ATEM D2 WebView | [ATEM SDI Extreme ISO][atem-sdi]
`0xbe9d` | ATEM D2 WebView | [ATEM Television Studio HD8][atem-tv]
`0xbe9e` | ATEM D2 WebView | [ATEM Television Studio HD8 ISO][atem-tv]
`0xbeba` | ATEM D2 | ATEM 2 M/E Advanced Panel 30
`0xbebb` | ATEM D2 | ATEM 2 M/E Advanced Panel 40
`0xbec8` | Mic Converter DFU | ATEM Microphone Converter
`0xbed7` | ATEM D2 WebView | [ATEM 1 M/E Constellation 4K][atem-constellation]
`0xbed8` | ATEM D2 WebView | [ATEM 2 M/E Constellation 4K][atem-constellation]
`0xbed9` | ATEM D2 WebView | [ATEM 4 M/E Constellation 4K][atem-constellation]
`0xbeda` | ATEM D2 WebView | [ATEM Television Studio 4K8][atem-tv]
`0xbedb` | IP Video 2 DFU | Blackmagic 2110 IP Converter 3x3G
`0xbee4` | ATEM D2 | ATEM 1 M/E Advanced Panel 20
`0xbee5` | ATEM D2 | ATEM 1 M/E Advanced Panel 30
`0xbef0` | ATEM Panels Micro | [ATEM Micro Panel][atem-micro-panel]
`0xbef3` | Micro Converters DFU | Blackmagic 2110 IP Mini IP to HDMI SFP
`0xbef4` | Micro Converters DFU | Blackmagic 2110 IP Mini BiDirect 12G SFP
`0xbef5` | Micro Converters DFU | Blackmagic 2110 IP Mini IP to HDMI
`0xbef6` | Micro Converters DFU | Blackmagic 2110 IP Mini BiDirect 12G
**`0xbf__`** | ... | ... 
`0xbf01` | ATEM Panels Micro | [ATEM Micro Camera Panel][atem-micro-camera-panel]

[adv-panel]: https://www.blackmagicdesign.com/products/atemconstellation/advancedpanel
[atem-8k]: https://www.blackmagicdesign.com/products/atemconstellation8k
[atem-cam]: https://www.blackmagicdesign.com/products/atemcameracontrolpanel
[atem-constellation]: https://www.blackmagicdesign.com/products/atemconstellation
[atem-micro-camera-panel]: https://www.blackmagicdesign.com/products/atemconstellation/techspecs/W-ABP-12
[atem-micro-panel]: https://www.blackmagicdesign.com/products/atemconstellation/techspecs/W-ABP-11
[atem-mini]: https://www.blackmagicdesign.com/products/atemmini
[atem-sdi]: https://www.blackmagicdesign.com/products/atemsdi
[atem-tv]: https://www.blackmagicdesign.com/products/atemtelevisionstudio
[micro-converter]: https://www.blackmagicdesign.com/products/microconverters
[mini-converter]: https://www.blackmagicdesign.com/products/miniconverters
[web-presenter]: https://www.blackmagicdesign.com/products/blackmagicwebpresenter
