use structopt::StructOpt;
use std::thread::sleep;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::time::Duration;
mod usb2snes;


#[derive(StructOpt, Debug)]
#[structopt(name = "usb2snes-cli", about = "usb2snes-cli --boot /games/Super Metroid.smc")]
struct Opt {
    #[structopt(long, help = "Operate in silent mode")]
    quiet:  Option<bool>,

    #[structopt(long, name = "list", help = "List the device available")]
    list_device: bool,
    #[structopt(long, name = "list-loop", help = "List the device every second")]
    list_device_loop: bool,

    #[structopt(long, help = "Use the specified device")]
    device: Option<String>,

    #[structopt(long, help = "Reset the game running on the device")]
    reset: bool,
    #[structopt(long, help = "Bring back the sd2snes/fxpak pro to the menu")]
    menu: bool,
    #[structopt(long, name = "File to boot", help = "Boot the specified file")]
    boot: Option<String>,
    
    #[structopt(long = "ls", name = "List the specified directory", help = "List the specified directory, path separator is /")]
    ls_path: Option<String>,

    #[structopt(long = "upload", name = "File to upload", help = "Upload a file to the device, use --path to specify the path on the device, like --upload SM.smc --path=/games/Super Metroid.smc")]
    file_to_upload: Option<String>,

    #[structopt(long = "path", name = "The path on the device")]
    path: Option<String>,

    #[structopt(long = "download", name = "File to download")]
    file_to_download: Option<String>,

    #[structopt(long = "devel", name = "Show all the transaction with the usb2snes server")]
    devel: bool
}

fn main() {
    let opt = Opt::from_args();

    let mut usb2snes;
    if opt.devel {
        usb2snes = usb2snes::usb2snes::SyncClient::connect_with_devel();
    } else {
        usb2snes = usb2snes::usb2snes::SyncClient::connect();
    }
    println!("Connected to the Usb2snes server");
    usb2snes.set_name(String::from("usb2snes-cli"));
    println!("Server version is : {:?}", usb2snes.app_version());

    let mut devices = usb2snes.list_device();

    if opt.list_device_loop {
        loop {
            println!("Devices : {:?}", devices);
            sleep(Duration::new(0, 5000000));
            devices = usb2snes.list_device();
        }
    }
    if devices.is_empty() {
        println!("No device found");
        std::process::exit(1);
    } else {
        if opt.list_device {
            println!("Listing devices :");
            for dev in devices {
                let mut infoclient = usb2snes::usb2snes::SyncClient::connect();
                infoclient.attach(&dev);
                let info = infoclient.info();
                println!("For device : {:?}, type : {:?}, version :{:?}, game : {:?} - Flags : {:?}", dev, info.dev_type, info.version, info.game, info.flags);
            }
            std::process::exit(0);
        }
        let device = opt.device.unwrap_or_else(||devices[0].clone());
        if !devices.contains(&device) {
            println!("Can't find the specified device <{:?}>", &device);
            std::process::exit(1);
        }
        usb2snes.attach(&device);
        let info = usb2snes.info();
        if info.flags.contains(&String::from("NO_FILE_CMD")) && (opt.file_to_download != None || opt.file_to_upload != None || opt.ls_path != None) {
            println!("The device does not support file commands");
            std::process::exit(1);
        }
        if info.flags.contains(&String::from("NO_CONTROL_CMD")) && (opt.menu || opt.reset || opt.boot != None) {
            println!("The device does not support control command (menu/reset/boot)");
            std::process::exit(1);
        }
        if opt.menu {
            usb2snes.menu();
        }
        if opt.boot != None {
            usb2snes.boot(&opt.boot.unwrap());
        }
        if opt.reset {
            usb2snes.reset();
        }
        if opt.ls_path != None {
            let path = opt.ls_path.unwrap().to_string();
            let dir_infos = usb2snes.ls(&path);
            println!("Listing {:?} : ", path);
            for info in dir_infos {
                println!("{:?}", format!("{}{}", info.name, if info.file_type == usb2snes::usb2snes::USB2SnesFileType::Dir {"/"} else {""}));
            }
        }
        if opt.file_to_upload != None {
            if opt.path == None {
                println!("You need to provide a --path to upload a file");
                std::process::exit(1);
            }
            let local_path = opt.file_to_upload.unwrap();
            let data = fs::read(local_path).expect("Error opening the file or reading the content");
            let path = opt.path.unwrap();
            usb2snes.send_file(&path, data);
        }
        if opt.file_to_download != None {
            let path:String = opt.file_to_download.unwrap();
            let local_path = path.split('/').last().unwrap();
            println!("Downloading : {:?} , local file {:?}", path, local_path);
            let data = usb2snes.get_file(&path);
            let f = File::create(local_path);
            let mut f = match f {
                Ok(file) => file,
                Err(err) => panic!("Probleme opening the file {:?} : {:?}", path, err),
            };
            f.write_all(&data);
        }
    }
}