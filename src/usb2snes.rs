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


pub mod usb2snes {

use websocket::{ClientBuilder, Message};
use websocket::sync::stream::TcpStream;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

    #[derive(Display, Debug)]
    #[allow(dead_code)]
    pub enum Command {
        AppVersion,
        Name,
        DeviceList,
        Attach,
        Info,
        Boot,
        Reset,
        Menu,

        List,
        PutFile,
        GetFile,
        Rename,
        Remove,

        GetAddress
    }
    #[derive(Display, Debug)]
    #[allow(dead_code)]
    pub enum Space {
        None,
        SNES,
        CMD
    }

    pub struct Infos {
        pub version : String,
        pub dev_type : String,
        pub game : String,
        pub flags : Vec<String>
    }

    #[derive(Serialize)]
    #[allow(non_snake_case)]
    struct USB2SnesQuery {
        Opcode: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        Space: Option<String>,
        Flags: Vec<String>,
        Operands: Vec<String>
    }
    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct USB2SnesResult {
        Results : Vec<String>
    }

    #[derive(PartialEq)]
    pub enum USB2SnesFileType {
        File = 0,
        Dir = 1
    }
    pub struct USB2SnesFileInfo {
        pub name : String,
        pub file_type : USB2SnesFileType
    }
    
    pub struct SyncClient {
        client : websocket::sync::Client<TcpStream>,
        devel : bool
    }
    impl SyncClient {
        pub fn connect() -> SyncClient {
            SyncClient {
                 client : ClientBuilder::new("ws://localhost:23074")
                .unwrap()
                .connect_insecure()
                .unwrap(),
                devel : false
            }
        }
        pub fn connect_with_devel() -> SyncClient {
            SyncClient {
                client : ClientBuilder::new("ws://localhost:23074")
                .unwrap()
                .connect_insecure()
                .unwrap(),
                devel : true
            }
        }
        fn send_command(&mut self, command : Command, args : Vec<String>) {
            self.send_command_with_space(command, None, args)
        }
        fn send_command_with_space(&mut self, command : Command, space : Option<Space>,  args : Vec<String>) {
            if self.devel {
                println!("Send command : {:?}", command);
            }
            let nspace : Option<String> = match space {
                None => None,
                Some(sp) => Some(sp.to_string())
            };
            let query = USB2SnesQuery {
                Opcode : command.to_string(),
                Space : nspace,
                Flags : vec![],
                Operands : args
            };
            let json = serde_json::to_string_pretty(&query).unwrap();
            if self.devel {
                println!("{}", json);
            }
            let message = Message::text(json);
            self.client.send_message(&message).unwrap();
        }

        fn get_reply(&mut self) -> USB2SnesResult {
            let reply = self.client.recv_message().unwrap();
            let mut textreply : String = String::from("");
            match reply
            {
                websocket::OwnedMessage::Text(value) => {textreply = value;}
                _ => {println!("Error getting a reply");}
            };
            if self.devel {
                println!("Reply:");
                println!("{}", textreply);
            }
            return serde_json::from_str(&textreply).unwrap();
        }
        pub fn set_name(&mut self, name : String) {
            self.send_command(Command::Name, vec![name]);
        }
        pub fn app_version(&mut self) -> String {
            self.send_command(Command::AppVersion, vec![]);
            let usbreply = self.get_reply();
            return usbreply.Results[0].to_string();
        }
        pub fn list_device(&mut self) -> Vec<String> {
            self.send_command(Command::DeviceList, vec![]);
            let usbreply = self.get_reply();
            return usbreply.Results;
        }
        pub fn attach(&mut self, device : &String) {
            self.send_command(Command::Attach, vec![device.to_string()]);
        }

        pub fn info(&mut self) -> Infos {
            self.send_command(Command::Info, vec![]);
            let usbreply = self.get_reply();
            let info : Vec<String> =  usbreply.Results;
            Infos { version: info[0].clone(), dev_type: info[1].clone(), game: info[2].clone(), flags: (info[3..].to_vec()) }
        }
        pub fn reset(&mut self) {
            self.send_command(Command::Reset, vec![]);
        }
        pub fn menu(&mut self) {
            self.send_command(Command::Menu, vec![]);
        }
        pub fn boot(&mut self, toboot : &String) {
            self.send_command(Command::Boot, vec![toboot.clone()]);
        }

        pub fn ls(&mut self, path : &String) -> Vec<USB2SnesFileInfo> {
            self.send_command(Command::List,vec![path.to_string()]);
            let usbreply = self.get_reply();
            let vec_info = usbreply.Results;
            let mut toret:Vec<USB2SnesFileInfo> = vec![];
            let mut i =  0;
            while i < vec_info.len() {
                let info : USB2SnesFileInfo = USB2SnesFileInfo {
                    file_type : if vec_info[i] == "1" {USB2SnesFileType::File} else {USB2SnesFileType::Dir},
                    name : vec_info[i + 1].to_string()
                };
                toret.push(info);
                i += 2;
            }
            return toret;
        }
        pub fn send_file(&mut self, path : &String, data : Vec<u8>) {
            self.send_command(Command::PutFile, vec![path.to_string(), format!("{:x}", data.len())]);
            let mut start = 0;
            let mut stop = 1024;
            while start < data.len() {
                self.client.send_message(&Message::binary(&data[start..stop])).unwrap();
                start += 1024;
                stop += 1024;
                if stop > data.len() {
                    stop = data.len();
                }
            }
        }
        pub fn get_file(&mut self, path : &String) -> Vec<u8> {
            self.send_command(Command::GetFile, vec![path.clone()]);
            let string_hex = self.get_reply().Results[0].to_string();
            let size = usize::from_str_radix(&string_hex.to_string(), 16).unwrap();
            let mut data : Vec<u8> = vec![];
            data.reserve(size);
            loop {
                let reply = self.client.recv_message().unwrap();
                match reply {
                    websocket::OwnedMessage::Binary(msgdata) => {data.extend(&msgdata);}
                    _ => {println!("Error getting a reply");}
                }
                if data.len() == size {
                    break;
                }
            }
            data
        }
        pub fn remove_path(&mut self, path : &String) {
            self.send_command(Command::Remove, vec![path.clone()]);
        }
        pub fn get_address(&mut self, address : u32, size : usize) -> Vec<u8> {
            self.send_command_with_space(Command::GetAddress, Some(Space::SNES), vec![format!("{:x}", address), format!("{:x}", size)]);
            let mut data : Vec<u8> = vec![];
            data.reserve(size);
            loop {
                let reply = self.client.recv_message().unwrap();
                match reply {
                    websocket::OwnedMessage::Binary(msgdata) => {data.extend(&msgdata);}
                    _ => {println!("Error getting a reply");}
                }
                if data.len() == size {
                    break;
                }
            }
            data
        }
    }

}