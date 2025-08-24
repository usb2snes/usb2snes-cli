/*
 * Copyright (c) 2021 Sylvain "Skarsnik" Colinet
 *
 * This file is part of the usb2snes-cli project.
 * (see https://github.com/usb2snes/usb2snes-cli).
 *
 * usb2snes-cli is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * usb2snes-cli is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with QUsb2Snes.  If not, see <https://www.gnu.org/licenses/>.
 */

use scan_fmt::scan_fmt;
use std::fs::File;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, fs};

use structopt::StructOpt;

use rusb2snes::{SyncClient, USB2SnesFileType};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "usb2snes-cli",
    about = "usb2snes-cli --boot /games/Super Metroid.smc"
)]
struct Opt {
    //#[structopt(long, help = "Operate in silent mode")]
    //quiet:  Option<bool>,
    #[structopt(long, name = "list", help = "List the device available")]
    list_device: bool,
    #[structopt(long, name = "list-loop", help = "List the device every second")]
    list_device_loop: bool,

    #[structopt(long, help = "Use the specified device")]
    device: Option<String>,

    #[structopt(
        long = "get-address",
        help = "Read a usb2snes address, syntax address_in_hex:size"
    )]
    get_address: Option<String>,

    #[structopt(long, help = "Reset the game running on the device")]
    reset: bool,
    #[structopt(long, help = "Bring back the sd2snes/fxpak pro to the menu")]
    menu: bool,
    #[structopt(long, name = "File to boot", help = "Boot the specified file")]
    boot: Option<String>,

    #[structopt(
        long = "ls",
        name = "List the specified directory",
        help = "List the specified directory, path separator is /"
    )]
    ls_path: Option<String>,

    #[structopt(
        long = "upload",
        name = "File to upload",
        help = "Upload a file to the device, use --path to specify the path on the device, like --upload SM.smc --path=/games/Super Metroid.smc"
    )]
    file_to_upload: Option<String>,

    #[structopt(long = "path", name = "The path on the device")]
    path: Option<String>,

    #[structopt(long = "download", name = "File to download")]
    file_to_download: Option<String>,

    #[structopt(long = "rm", name = "Path on the device of a file to remove")]
    path_to_remove: Option<String>,

    #[structopt(
        long = "devel",
        name = "Show all the transaction with the usb2snes server"
    )]
    devel: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let mut usb2snes;
    if opt.devel {
        usb2snes = SyncClient::connect_with_devel()?;
    } else {
        usb2snes = SyncClient::connect()?;
    }
    println!("Connected to the Usb2snes server");
    usb2snes.set_name(String::from("usb2snes-cli"))?;
    println!("Server version is : {}", usb2snes.app_version()?);

    let mut devices = usb2snes.list_device()?;

    if opt.list_device_loop {
        loop {
            println!("Devices : {:?}", devices);
            sleep(Duration::new(0, 5000000));
            devices = usb2snes.list_device()?;
        }
    }
    if devices.is_empty() {
        return Err("No Device Found.".into());
    } else {
        if opt.list_device {
            println!("Listing devices :");
            for dev in devices {
                let mut infoclient = SyncClient::connect()?;
                infoclient.attach(&dev)?;
                let info = infoclient.info()?;
                println!(
                    "For device : {:?}, type : {:?}, version :{:?}, game : {:?} - Flags : {:?}",
                    dev, info.dev_type, info.version, info.game, info.flags
                );
            }
            return Ok(());
        }

        let device = match opt.device {
            Some(d) => d,
            None => return Err("Error parsing device".into()),
        };
        // let device = opt.device.unwrap_or_else(|| devices[0].clone());
        if !devices.contains(&device) {
            return Err(format!("Can't find the specified device <{:?}>", &device).into());
        }
        usb2snes.attach(&device)?;
        let info = usb2snes.info()?;
        if info.flags.contains(&String::from("NO_FILE_CMD"))
            && (opt.file_to_download.is_some()
                || opt.file_to_upload.is_some()
                || opt.ls_path.is_some())
        {
            return Err("The device does not support file commands".into());
        }
        if info.flags.contains(&String::from("NO_CONTROL_CMD"))
            && (opt.menu || opt.reset || opt.boot.is_some())
        {
            return Err("The device does not support control command (menu/reset/boot)".into());
        }
        if opt.get_address.is_some() {
            let toget = opt.get_address.unwrap();
            if let Ok((address, size)) = scan_fmt!(&toget, "{x}:{d}", [hex u32], usize) {
                let data = usb2snes.get_address(address, size)?;
                let mut i = 0;
                while i < data.len() {
                    if i % 16 == 0 {
                        println!();
                        print!("{:02X} : ", i);
                    }
                    print!("{:02X} ", data[i]);
                    i += 1;
                }
            }
        }
        if opt.menu {
            usb2snes.menu()?;
        }
        if opt.boot.is_some() {
            usb2snes.boot(&opt.boot.unwrap())?;
        }
        if opt.reset {
            usb2snes.reset()?;
        }
        if opt.ls_path.is_some() {
            let path = opt.ls_path.unwrap();
            let dir_infos = usb2snes.ls(&path)?;
            println!("Listing {:?} : ", path);
            for info in dir_infos {
                println!(
                    "{:?}",
                    format!(
                        "{}{}",
                        info.name,
                        if info.file_type == USB2SnesFileType::Dir {
                            "/"
                        } else {
                            ""
                        }
                    )
                );
            }
        }
        if opt.file_to_upload.is_some() {
            if opt.path.is_none() {
                return Err("You need to provide a --path to upload a file".into());
            }
            let local_path = opt.file_to_upload.unwrap();
            let data = fs::read(local_path).expect("Error opening the file or reading the content");
            let path = opt.path.unwrap();
            usb2snes.send_file(&path, data)?;
        }
        if opt.file_to_download.is_some() {
            let path: String = match opt.file_to_download {
                Some(p) => p,
                None => return Err("File Not Found".into()),
            };
            let local_path = match path.split('/').next_back() {
                Some(p) => p,
                None => return Err("Could not parse local_path.".into()),
            };
            println!("Downloading : {:?} , local file {:?}", path, local_path);
            let data = usb2snes.get_file(&path)?;
            let f = File::create(local_path);
            let mut f = match f {
                Ok(file) => file,
                Err(err) => panic!("Problem opening the file {:?} : {:?}", path, err),
            };
            f.write_all(&data)
                .expect("Can't write the data to the file");
        }
        if opt.path_to_remove.is_some() {
            let path: String = opt.path_to_remove.unwrap();
            println!("Removing : {:?}", path);
            usb2snes.remove_path(&path)?;
        }
    }
    Ok(())
}
