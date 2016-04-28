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
#[allow(unused_must_use)]
fn request_respond_string() {
    let mut ctx: zmq::Context = zmq::Context::new();

    // run requester (client)
    let mut req_socket = ctx.socket(zmq::REQ).unwrap();
    let req_handle = thread::spawn(move || {
        req_socket.connect("inproc://zmq-test");
        req_socket.send_str("hello", 0);
        let s = req_socket.recv_string(0).unwrap().unwrap();
        assert_eq!("world", s);
        req_socket.close();
    });

    // run responder (server)
    // let mut ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("inproc://zmq-test");
    let s = socket.recv_string(0).unwrap().unwrap();
    assert_eq!("hello", s);
    socket.send_str("world", 0);
    req_handle.join();
}

#[test]
#[allow(unused_must_use)]
fn request_respond_msg() {
    static REQ_DATA: TestStruct = TestStruct { x: 1 };
    static RESP_DATA: TestStruct = TestStruct { x: 2 };
    let mut ctx = zmq::Context::new();
    let bin_limit = bincode::SizeLimit::Bounded(4);

    // run requester (client)
    let mut req_socket = ctx.socket(zmq::REQ).unwrap();
    let req_handle = thread::spawn(move || {
        // setup
        req_socket.connect("inproc://zmq-test");
        // send req
        let bin_data = encode(&REQ_DATA, bin_limit).unwrap();
        let msg_out = zmq::Message::from_slice(&bin_data).unwrap();
        req_socket.send_msg(msg_out, 0);
        // receive resp
        let mut msg_in = zmq::Message::new().unwrap();
        req_socket.recv(&mut msg_in, 0).unwrap();
        let resp: TestStruct = decode(&msg_in).unwrap();
        assert_eq!(RESP_DATA, resp);
        // cleanup
        req_socket.close();
    });

    // run responder (server)
    // setup
    let mut socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("inproc://zmq-test");
    // receive req
    let msg_in = socket.recv_bytes(0).unwrap();
    let req: TestStruct = decode(&msg_in[..]).unwrap();
    assert_eq!(REQ_DATA, req);
    // send resp
    let bin_data = encode(&RESP_DATA, bin_limit).unwrap();
    let msg_out = zmq::Message::from_slice(&bin_data).unwrap();
    socket.send_msg(msg_out, 0);

    // cleanup
    req_handle.join();
}
