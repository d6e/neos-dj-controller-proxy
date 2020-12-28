#[macro_use]
extern crate lazy_static;
mod midi;

use midir::MidiOutputConnection;
use regex::Regex;
use std::borrow::BorrowMut;
use std::net::SocketAddr;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

fn main() {
    ws_server()
}

lazy_static! {
    static ref PARSE_MIDI: Regex =
        Regex::new(r"^\[(?P<first>\d{1,3}),(?P<second>\d{0,3}),(?P<third>\d{0,3})\]$").unwrap();
}

fn ws_server() {
    let addr0 = SocketAddr::from(([0, 0, 0, 0], 8888));
    let server = Server::bind(addr0).unwrap();
    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        let mut client = request.accept().unwrap();
        let ip = client.peer_addr().unwrap();

        println!("Connection from {}", ip);

        let mut midi_conn = midi::connect().unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        for message in receiver.incoming_messages() {
            let message = message.unwrap();

            match message {
                OwnedMessage::Close(_) => {
                    let message = OwnedMessage::Close(None);
                    sender.send_message(&message).unwrap();
                    println!("Client {} disconnected", ip);
                    return;
                }
                OwnedMessage::Ping(ping) => {
                    println!("Ping={:?}", ping);
                }
                OwnedMessage::Text(msg) => {
                    println!("Message: {}", msg);
                    let msg = msg.trim();
                    if msg.starts_with("[") {
                        let midi: [u8; 3] = PARSE_MIDI
                            .captures(msg)
                            .map(|x| {
                                [
                                    x.name("first").unwrap().as_str().parse::<u8>().unwrap(),
                                    x.name("second").unwrap().as_str().parse::<u8>().unwrap(),
                                    x.name("third").unwrap().as_str().parse::<u8>().unwrap(),
                                ]
                            })
                            .unwrap();
                        midi_conn.send(&midi);
                    } else {
                        println!(
                            "Message not valid midi: {} \nMust be in format [0,0,0].",
                            msg
                        );
                    }
                }
                OwnedMessage::Binary(data) => {
                    println!("Binary: {:?}", data);
                }
                OwnedMessage::Pong(data) => {
                    println!("Pong: {:?}", data);
                } // _ => sender.send_message(&message).unwrap(),
            }
        }
    }
}
