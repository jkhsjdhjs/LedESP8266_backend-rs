extern crate rustc_serialize;
extern crate hyper;

use hyper::server::{Server, Request, Response};
use hyper::header::ContentType;
use hyper::mime;
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
fn handler(mut req: Request, mut res: Response) {
    res.headers_mut().set(ContentType(mime::Mime(mime::TopLevel::Application,
                                                 mime::SubLevel::Json,
                                                 vec![(mime::Attr::Charset, mime::Value::Utf8)])));
    println!("New request");
    let mut buf = Vec::with_capacity(4);
    let mut payload = String::with_capacity(128);
    if let Err(e) = req.read_to_string(&mut payload) {
        *res.status_mut() = hyper::status::StatusCode::BadRequest;
        res.send(format!(r#"{{ "err": "Invalid Body", "msg": "{}" }}"#, e).as_bytes()).unwrap();
        return;
    };
    if !is_valid_payload(&payload) {
        *res.status_mut() = hyper::status::StatusCode::BadRequest;
        res.send(br#"{ "err": "Invalid Json" }"#).unwrap();
        return;
    };
    {
        let mut socket = TcpStream::connect("192.168.0.26:6550").unwrap();
        socket.write(payload.as_bytes()).unwrap(); // forward input to socket
        socket.read(&mut buf).unwrap();
    }
    res.send(&buf).unwrap();
}

fn main() {
    Server::http("0.0.0.0:8080").unwrap().handle(handler).unwrap();
    println!("Listening on 0.0.0.0:8080");
}
