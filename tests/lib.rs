extern crate zmq;

use std::thread;

#[test]
fn receive_respond() {
    // run requester (client)
    let req_handle = thread::spawn(|| {
        let mut ctx = zmq::Context::new();
        let mut socket = ctx.socket(zmq::REQ).unwrap();
        socket.connect("tcp://localhost:12345");
        socket.send_str("hello", 0);
        let s = socket.recv_string(0).unwrap().unwrap();
        assert_eq!("world", s);
        socket.close();
    });

    // run responder (server)
    let mut ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::REP).unwrap();
    socket.bind("tcp://*:12345");
    let s = socket.recv_string(0).unwrap().unwrap();
    assert_eq!("hello", s);
    socket.send_str("world", 0);
    req_handle.join();
}
