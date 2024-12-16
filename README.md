# Blackmagic firmware reverse engineering tools

This repository contains tools for reverse engineering firmware for Blackmagic
Design devices (ATEM, Web Presenter, etc.)

There are two parts:

* `libbmfw`: library for parsing firmware files
* `bmfw`: CLI tool for inspecting and extracting firmware files

As this software is for reverse engineering undocumented file formats and has a
pre-1.0 version label, its APIs, interfaces and MSRV should be considered
*unstable* and *subject to change at any time*.

## Getting started

There are no official binary builds of this software, and you'll need to
[install a *recent* Rust compiler toolchain with `rustup`][install-rust].

[install-rust]: https://www.rust-lang.org/tools/install

You can then build everything with:

```sh
cargo build --release
```

The binary will be in `target/release/bmfw` or `target/release/bmfw.exe`.

You can then extract some firmware with:

```sh
./target/release/bmfw extract /path/to/data-bde6.bin -o /tmp/bde6
```

## Finding firmware

Firmware files named `data-${PID}.bin`, matching the device's USB product
ID[^wpid] in lower case.

Some devices like the ATEM Television Studio have extra firmware files for UI
translations, which are suffixed with a language code.

Firmware files can be found in a few places:

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

[^wpid]: For the Web Presenter, this is the product ID of the front-panel
USB port (for firmware updates).

## Extract firmware from packages without installing

There are instructions for [macOS](#extract-macos-installers) and
[Windows](#extract-windows-installers) installers.

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
To find files named like firmware images (`data-[0-9a-f]{4}(-[^/]+)?\.bin`):

```sh
7z l Payload | egrep "data-[0-9a-f]{4}(-[^/]+)?\\.bin"
```

Entries will have the full path on disk where it'd be normally installed.

Once you've found the firmware file(s) you're interested in, extract with:

```sh
7z e Payload "./Applications/....app/Contents/Resources/data-ABCD.bin
```

### Extract Windows installers

Blackmagic software on Windows is distributed as a ZIP file containing an `.msi`
file, and sometimes also with one or more *hidden* [`.cab` files][cab]. All
files can be extracted with the 7-Zip GUI or its CLI (`7z`).

[cab]: https://en.wikipedia.org/wiki/Cabinet_(file_format)

You can check if a `.cab` contains firmware by looking for files named like
firmware images with a shell script:

```sh
7z l ExampleSupportFile.cab | egrep "data_[0-9a-f]{4}(_[^/]+)?\\.bin"
```

For `.msi` files, the file names are a litle bit different:

```sh
7z l ExampleInstaller.msi | egrep -i "Data[0-9a-f]{4}"
```

You can extract these using similar steps to the
[macOS instructions](#extract-macos-installers) (above)

Doing this with PowerShell is left as an exercise for the reader (it's not
trivial). 🙂

## Data structure descriptions

There are notes for each data structure available as
[`rustdoc` comments](https://doc.rust-lang.org/rustdoc/). You can access these
via:

* [docs.rs of the latest tagged release](https://docs.rs/libbmfw/latest/libbmfw/)
* Build the current docs locally with `cargo doc -p libbmfw`
* Read source file comments starting with `///` in `./libbmfw/src`

## Firmware structure

Blackmagic firmware files consist of one or more resources:

* FPGA gateware, which is used for SDI/HDMI capture/switching and runs a
  MicroBlaze (or PowerPC?) softcore
* ELF with software for the MicroBlaze or PowerPC core
* `tar` archive containing non-executable resources (eg: fonts, images, strings)
* `tar` archive containing (encrypted?) gateware / software
