
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use std::net::SocketAddr;

fn main() {
    let addr1 = SocketAddr::from(([0, 0, 0, 0], 8888));
    let server = Server::bind(addr1).unwrap();
    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            let mut client = request.accept().unwrap();
            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

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
                        println!("Ping: {:?}", ping );
                    }
                    OwnedMessage::Text(msg) => {
                        println!("Message: {}", msg);
                    }
                    OwnedMessage::Binary(data) => {
                        println!("Binary: {:?}", data);
                    }
                    OwnedMessage::Pong(data) => {
                        println!("Pong: {:?}", data);
                    }
                    // _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}