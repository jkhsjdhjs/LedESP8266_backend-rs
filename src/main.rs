extern crate ws;
extern crate uuid;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::thread;
use std::sync::Mutex;
use std::collections::HashMap;
use std::cell::RefCell;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Message {
    caller: Option<String>,
    #[serde(rename = "type")]
    type_name: String,
    msg: String,
    data: Option<Data>
}

#[derive(Serialize, Deserialize)]
struct Data {
    dev: Option<u8>,
    reg: Option<u8>,
    data: Option<u8>,
    color: Option<Color>,
    fade_time: Option<u64>
}

#[derive(Serialize, Deserialize, Clone)]
struct Color {
    red: u16,
    green: u16,
    blue: u16
}

struct WSModuleConnection {
    uuid: String,
    out: ws::Sender
}

impl ws::Handler for WSModuleConnection {
    fn on_open(&mut self, _: ws::Handshake) -> Result<(), ws::Error> {
        println!("MODULES: Connection with UUID: {} opened.", self.uuid);
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.modules.lock().unwrap();
            let mut d = f.borrow_mut();
            d.insert(self.uuid.clone(), self.out.clone());
        }
        Ok(())
    }
    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        let msg_str: String = msg.into_text().unwrap();
        println!("MODULES: Message from connection with UUID: {}:\n{}", self.uuid, msg_str);
        let result: Result<Message, serde_json::Error> = serde_json::from_str(&msg_str);
        if result.is_err() {
            eprintln!("MODULES: Failed to decode message!\n");
            return Ok(());
        }
        let msg_decoded = result.unwrap();
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.website.lock().unwrap();
            let d = f.borrow();
            match msg_decoded.data {
                Some(ref x) => {
                    match x.color {
                        Some(ref a) => {
                            let z = Message {
                                caller: None,
                                type_name: msg_decoded.type_name,
                                msg: msg_decoded.msg,
                                data: Some(Data {
                                    color: Some(Color {
                                        red: a.red,
                                        green: a.green,
                                        blue: a.blue
                                    }),
                                    fade_time: x.fade_time,
                                    dev: None,
                                    reg: None,
                                    data: None
                                })
                            };
                            match msg_decoded.caller {
                                None => {
                                    for val in d.values() {
                                        let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                                        println!("MODULE: Sending to website: {}", json);
                                        if val.send(json).is_ok() {
                                            println!("sending succeeded!");
                                        }
                                        else {
                                            eprintln!("sending failed!");
                                        }
                                    }
                                },
                                Some(ref caller) => {
                                    let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                                    println!("MODULE: Sending to website: {}", json);
                                    d.get(caller).unwrap().send(json).unwrap();
                                }
                            };
                        },
                        None => {
                            let z = Message {
                                caller: None,
                                type_name: msg_decoded.type_name,
                                msg: msg_decoded.msg,
                                data: Some(Data {
                                    dev: x.dev,
                                    reg: x.reg,
                                    data: x.data,
                                    color: None,
                                    fade_time: None
                                })
                            };
                            match msg_decoded.caller {
                                None => {
                                    for val in d.values() {
                                        let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                                        println!("MODULE: Sending to website: {}", json);
                                        val.send(json).unwrap();
                                    }
                                },
                                Some(ref caller) => {
                                    let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                                    println!("MODULE: Sending to website: {}", json);
                                    d.get(caller).unwrap().send(json).unwrap();
                                }
                            };
                        }
                    }
                },
                None => { }
            };
        }
        Ok(())
    }
    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        let c: u16 = code.into();
        println!("MODULES: Connection with UUID: {} closed. Code: {}. Reason: {}", self.uuid, c, reason);
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.modules.lock().unwrap();
            let mut d = f.borrow_mut();
            d.remove(&self.uuid.clone());
        }
    }
}

impl WSModuleConnection {
    fn new(out: ws::Sender) -> Self {
        WSModuleConnection {
            uuid: Uuid::new_v4().to_hyphenated().to_string(),
            out: out
        }
    }
}

struct WSWebsiteConnection {
    uuid: String,
    out: ws::Sender
}

impl ws::Handler for WSWebsiteConnection {
    fn on_open(&mut self, _: ws::Handshake) -> Result<(), ws::Error> {
        println!("WEBSITE: Connection with UUID: {} opened.", self.uuid);
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.website.lock().unwrap();
            let mut d = f.borrow_mut();
            d.insert(self.uuid.clone(), self.out.clone());
        }
        Ok(())
    }
    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        let msg_str: String = msg.into_text().unwrap();
        println!("WEBSITE: Message from connection with UUID: {}:\n{}", self.uuid, msg_str);
        let result: Result<Message, serde_json::Error> = serde_json::from_str(&msg_str);
        if result.is_err() {
            eprintln!("WEBSITE: Failed to decode message!\n");
            return Ok(());
        }
        let msg_decoded = result.unwrap();
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.modules.lock().unwrap();
            let d = f.borrow();
            match msg_decoded.data {
                Some(ref x) => {
                    if msg_decoded.type_name == "command" && msg_decoded.msg == "set" {
                        let z = Message {
                            caller: Some(self.uuid.clone()),
                            type_name: msg_decoded.type_name,
                            msg: msg_decoded.msg,
                            data: Some(Data {
                                color: x.color.clone(),
                                fade_time: x.fade_time,
                                data: None,
                                dev: None,
                                reg: None
                            })
                        };
                        for val in d.values() {
                            let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                            println!("WEBSITE: Sending to modules: {}", json);
                            val.send(json).unwrap();
                        }
                    }
                },
                None => {
                    if msg_decoded.type_name == "command" && msg_decoded.msg == "get" {
                        let z = Message {
                            caller: Some(self.uuid.clone()),
                            type_name: msg_decoded.type_name,
                            msg: msg_decoded.msg,
                            data: None
                        };
                        for val in d.values() {
                            let json = serde_json::to_string(&serde_json::to_value(&z).unwrap()).unwrap();
                            println!("WEBSITE: Sending to modules: {}", json);
                            val.send(json).unwrap();
                        }
                    }
                }
            };
        }
        Ok(())
    }
    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        let c: u16 = code.into();
        println!("WEBSITE: Connection with UUID: {} closed. Code: {}. Reason: {}", self.uuid, c, reason);
        unsafe {
            let ref b = WS_CONNECTIONS;
            let d = WsConnections::new();
            let ref e = d;
            let a = match *b {
                Some(ref c) => { c },
                None => { e }
            };
            let f = a.website.lock().unwrap();
            let mut d = f.borrow_mut();
            d.remove(&self.uuid.clone());
        }
    }
}

impl WSWebsiteConnection {
    fn new(out: ws::Sender) -> Self {
        WSWebsiteConnection {
            uuid: Uuid::new_v4().to_hyphenated().to_string(),
            out: out
        }
    }
}

struct WsConnections {
    website: Mutex<RefCell<HashMap<String, ws::Sender>>>,
    modules: Mutex<RefCell<HashMap<String, ws::Sender>>>
}

impl WsConnections {
    fn new() -> Self {
        WsConnections {
            website: Mutex::new(RefCell::new(HashMap::new())),
            modules: Mutex::new(RefCell::new(HashMap::new()))
        }
    }
}

static mut WS_CONNECTIONS: Option<WsConnections> = None;

fn main() {
    unsafe {
        WS_CONNECTIONS = Some(WsConnections::new());
    }

    let modules = thread::spawn(move || {
        ws::listen("0.0.0.0:14795", |out| { WSModuleConnection::new(out) }).unwrap()
    });

    let website_backend = thread::spawn(move || {
        ws::listen("[::]:14796", |out| { WSWebsiteConnection::new(out) }).unwrap()
    });

    let _ = modules.join();
    let _ = website_backend.join();
}
