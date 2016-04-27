extern crate zmq;
extern crate bincode;
extern crate rustc_serialize;

use std::thread;
use bincode::rustc_serialize::{encode, decode};

#[derive(Debug, PartialEq, RustcDecodable, RustcEncodable)]
struct TestStruct {
    x: i32,
}

#[test]
fn struct_serialization() {
    // dumb test to make sure TestStruct serialization
    // isn't the problem for other failing tests
    let s = TestStruct { x: 1 };
    let encoded = encode(&s, bincode::SizeLimit::Infinite).unwrap();
    assert_eq!(encoded.len(), 4);  // 4 bytes for i32
    let decoded: TestStruct = decode(&encoded[..]).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn version_reporting() {
    let version = zmq::version();
    // use with cargo test -- --nocapture to verify output on your system
    println!("zmq version: {}.{}.{}", version.0, version.1, version.2);
    // assert_eq!(4, version.0);
    // assert_eq!(1, version.1);
    // assert_eq!(4, version.2);
}

#[test]
fn request_respond_string() {
    // run requester (client)
    let req_handle = thread::spawn(|| {
        let mut ctx = zmq::Context::new();
        let mut socket = ctx.socket(zmq::REQ).unwrap();
        let _ = socket.connect("tcp://localhost:12345").unwrap();
        let _ = socket.send_str("hello", 0).unwrap();
        let s = socket.recv_string(0).unwrap().unwrap();
        assert_eq!("world", s);
        let _ = socket.close().unwrap();
    });

    // run responder (server)
    let mut ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::REP).unwrap();
    let _ = socket.bind("tcp://*:12345").unwrap();
    let s = socket.recv_string(0).unwrap().unwrap();
    assert_eq!("hello", s);
    let _ = socket.send_str("world", 0).unwrap();
    let _ = req_handle.join().unwrap();
}

#[test]
fn request_respond_msg() {
    static req_data: TestStruct = TestStruct { x: 1 };
    static resp_data: TestStruct = TestStruct { x: 2 };
    let bin_limit = bincode::SizeLimit::Bounded(4);

    // run requester (client)
    let req_handle = thread::spawn(move || {
        // setup
        let mut ctx = zmq::Context::new();
        let mut socket = ctx.socket(zmq::REQ).unwrap();
        let _ = socket.connect("tcp://localhost:12346").unwrap();
        // send req
        let bin_data = encode(&req_data, bin_limit).unwrap();
        let msg_out = zmq::Message::from_slice(&bin_data).unwrap();
        let _ = socket.send_msg(msg_out, 0).unwrap();
        // receive resp
        let mut msg_in = zmq::Message::new().unwrap();
        socket.recv(&mut msg_in, 0).unwrap();
        let resp: TestStruct = decode(&msg_in).unwrap();
        assert_eq!(resp_data, resp);
        // cleanup
        let _ = socket.close().unwrap();
    });

    // run responder (server)
    // setup
    let mut ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::REP).unwrap();
    let _ = socket.bind("tcp://*:12346").unwrap();
    // receive req
    let msg_in = socket.recv_bytes(0).unwrap();
    let req: TestStruct = decode(&msg_in[..]).unwrap();
    // send resp
    let bin_data = encode(&resp_data, bin_limit).unwrap();
    let msg_out = zmq::Message::from_slice(&bin_data).unwrap();
    socket.send_msg(msg_out, 0);

    // cleanup
    let _ = req_handle.join().unwrap();
}
