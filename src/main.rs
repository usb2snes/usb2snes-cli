use std::{net::TcpStream, ops::RangeBounds, path::PathBuf};
use structopt::StructOpt;
mod usb2snes;


#[derive(StructOpt, Debug)]
#[structopt(name = "usb2snes-cli", about = "usb2snes-cli --boot /games/Super Metroid.smc")]
struct Opt {
    #[structopt(long, help = "Operate in silent mode")]
    quiet:  Option<bool>,

    #[structopt(long, name = "list", help = "List the device available")]
    list_device: bool,
    #[structopt(long, help = "Use the specified device")]
    device: Option<String>,

    #[structopt(long, help = "Reset the game running on the device")]
    reset: bool,
    #[structopt(long, help = "Bring back the sd2snes/fxpak pro to the menu")]
    menu: bool,
    #[structopt(long, name = "File to boot", help = "Boot the specified file")]
    boot: Option<String>,
    
    #[structopt(long = "upload", name = "File to upload", parse(from_os_str))]
    filetoupload: Option<PathBuf>,

    #[structopt(long = "download", name = "File to download")]
    filetodonwload: Option<String>
}

fn main() {
    let opt = Opt::from_args();

    let mut usb2snes = usb2snes::usb2snes::SyncClient::connect();
    println!("Connected to the Usb2snes server");
    usb2snes.setName(String::from("usb2snes-cli"));
    println!("Server version is : {:?}", usb2snes.appVersion());

    let devices = usb2snes.listDevice();
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
                println!("For device : {:?}, type : {:?}, version :{:?}, game : {:?} - Flags : {:?}", dev, info.devType, info.version, info.game, info.flags);
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
        if info.flags.contains(&String::from("NO_FILE_CMD")) && (opt.filetodonwload != None || opt.filetoupload != None) {
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
        if opt.filetodonwload != None {
            
        }
    }
}
