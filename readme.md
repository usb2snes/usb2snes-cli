This is a command line tool to interact with a SNES over USB, for example to transfer files to a flash cart's SD card. It has not been very thoroughly tested, so make sure you have backups of any important files on your SNES before using this.

It requires you to be running a separate program to manage the SNES communication. You can use either: 

* [QUsb2Snes](https://skarsnik.github.io/QUsb2snes/)
* [SNI](https://github.com/alttpo/sni) (also included as part of Archipelago) 

## Installation
If you're on windows, get it from the sidebar on the right of this github page.

Otherwise build it from source.

## Usage

`usb2snes-cli --help` for a description of all the available operations

Things you can do include: listing/writing/deleting files on the device, launching a specific rom, resetting the current
game, and returning to the main menu.

### upload-latest-sfc
This command lets you automatically select the newest .sfc file in some directory and send it to the device.
For example, you might use it like this:

`usb2snes-cli --upload-latest-sfc c:/users/<username>/Downloads incoming --wipe-target-dir`

This will take the newest sfc file in your Downloads directory and send it to the `incoming/` directory on the SNES. By including --wipe-target-dir, it makes it delete any other sfc files in `incoming/`

You might find it convenient to have a shortcut that does this if you play a lot of randos and you want a quick one-button way to get your newest rando rom onto your console.

## How to Build From Source
Install Rust from https://www.rust-lang.org/

Then do `cargo build`