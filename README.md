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

Firmware files named `data-[0-9a-f]{4}.bin`, matching the device's USB product
ID[^wpid].

Some devices like the ATEM Television Studio have extra firmware files for UI
translations.

Firmware files can be found in a few places:

* macOS:
  * `/Applications/Blackmagic ATEM Switchers/ATEM Setup.app/Contents/Resources`
  * `/Library/Application Supports/Blackmagic Design/*/AdminUtility/Plugins/*/Resources`
* Windows: `Program Files/Blackmagic Design/*/Setup/AdminUtility/Plugins/*/Resources`

You can find which firmware versions are included with a Blackmagic setup tool
via its debug window: hold the <kbd>Shift</kbd> key and double-left-click the
lower left corner of the window where you have a normal mouse cursor.

The debug window *cannot* be accessed by pressing <kbd>Tab</kbd> or using
accessibility tools.

[^wpid]: For the Web Presenter, this is the product ID of the front-panel
USB port (for firmware updates).

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
