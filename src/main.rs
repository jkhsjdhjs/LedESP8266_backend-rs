#[macro_use(itry)]
extern crate iron;
extern crate rustc_serialize;
use iron::prelude::*;
use iron::status;
use iron::headers::ContentType;
use iron::modifiers::Header;
use std::io::prelude::*;
use std::net::TcpStream;
use rustc_serialize::json;

#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable)]
struct Color {
    red: u16, // apparently u can have more than 255
    green: u16,
    blue: u16,
    fade_time: u64,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
struct ColorsReq {
    reqtype: String,
    data: Option<Color>,
}

fn is_valid_payload(payload: &str) -> bool {
    json::decode::<ColorsReq>(payload)
        .ok()
        .and_then(|mut s| {
            s.reqtype = s.reqtype.to_lowercase();
            match s {
                ref s if s.reqtype == "get" => Some(()),
                ref s if s.reqtype == "set" && s.data.is_some() => Some(()),
                _ => None,
            }
        })
        .is_some()
}

#[test]
fn test_valid_get_payload() {
    let payload = r#"{ "reqtype": "get" }"#;
    assert!(is_valid_payload(&payload));
}

#[test]
fn test_valid_set_payload() {
    let payload = r#"{
            "reqtype": "set",
            "data": {
                "red": 255,
                "green": 255,
                "blue": 255,
                "fade_time": 1000
            }
        }"#;
    assert!(is_valid_payload(&payload));
}
#[test]
fn test_invalid_set_payload() {
    let payload = r#"{ "reqtype": "set" }"#;
    assert!(!is_valid_payload(&payload))
}

/// Gets json from client and forwards it to the LED controls
/// echos the return from the LEDs back
fn handler(req: &mut Request) -> IronResult<Response> {
    println!("New request");
    let mut buf = String::with_capacity(4);
    let mut payload = String::with_capacity(128);
    itry!(req.body.read_to_string(&mut payload)); // get Input from Client
    if !is_valid_payload(&payload) {
        return Ok(Response::with((status::BadRequest,
                                  Header(ContentType::json()),
                                  r#"{ "err": "Invalid JSON" }"#)));
    }
    println!("Payload: {}", payload);
    {
        println!("connecting to esp...");
        let mut socket = TcpStream::connect("192.168.0.26:6550").unwrap();
        println!("connected.");
        itry!(socket.write(payload.as_bytes())); // forward input to socket
        itry!(socket.read_to_string(&mut buf));
    }
    println!("Buffer: {}", buf);
    // Just return buf for now:
    //

    Ok(Response::with((status::Ok, Header(ContentType::json()), buf)))
}

fn main() {
    let _server = Iron::new(handler).http("0.0.0.0:8080").unwrap();
    println!("Listening on 0.0.0.0:8080");
}
