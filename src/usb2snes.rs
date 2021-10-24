pub mod usb2snes {

use websocket::{ClientBuilder, Message, OwnedMessage};
use websocket::sync::stream::TcpStream;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

    #[derive(Display, Debug)]
    pub enum Command {
        AppVersion,
        Name,
        DeviceList,
        Attach,
        Info,
        Boot,
        Reset,
        Menu,

        PutFile,
        GetFile,
        Rename,
        Remove
    }
    pub enum Space {
        None,
        SNES,
        CMD
    }

    pub struct Infos {
        pub version : String,
        pub devType : String,
        pub game : String,
        pub flags : Vec<String>
    }

    #[derive(Serialize)]
    struct USB2SnesQuery {
        Opcode: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        Space: Option<String>,
        Flags: Vec<String>,
        Operands: Vec<String>
    }
    #[derive(Deserialize)]
    struct USB2SnesResult {
        Results : Vec<String>
    }
    
    
    pub struct SyncClient {
        client : websocket::sync::Client<TcpStream>,
    }
    impl SyncClient {
        pub fn connect() -> SyncClient {
            SyncClient {
                 client : ClientBuilder::new("ws://localhost:23074")
                .unwrap()
                .connect_insecure()
                .unwrap()
            }
        }
        fn send_command(&mut self, command : Command, args : Vec<String>) {
            println!("Send command : {:?}", command);
            let query = USB2SnesQuery {
                Opcode : command.to_string(),
                Space : None,
                Flags : vec![],
                Operands : args
            };
            let json = serde_json::to_string(&query).unwrap();
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
            return serde_json::from_str(&textreply).unwrap();
        }
        pub fn setName(&mut self, name : String) {
            self.send_command(Command::Name, vec![name]);
        }
        pub fn appVersion(&mut self) -> String {
            self.send_command(Command::AppVersion, vec![]);
            let usbreply = self.get_reply();
            return usbreply.Results[0].to_string();
        }
        pub fn listDevice(&mut self) -> Vec<String> {
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
            Infos { version: info[0].clone(), devType: info[1].clone(), game: info[2].clone(), flags: (info[3..].to_vec()) }
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

        pub fn send_file(&mut self, path : &String, data : Vec<u8>) {
            self.send_command(Command::PutFile, vec![path.clone()]);
            let mut start = 0;
            let mut stop = 1024;
            while stop <= data.len() {
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
            let size = self.get_reply().Results[0].parse::<usize>().unwrap();
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
            return data;
        }
    }

}