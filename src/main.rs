use std::net::UdpSocket;

fn main() {
    println!("Hello, world!");

    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Couldn't bind to address");
    let buffer = [1, 2, 3];
    socket
        .send_to(&buffer, "192.168.12.1:53")
        .expect("Couldn't send");
}
