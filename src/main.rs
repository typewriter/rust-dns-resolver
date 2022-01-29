mod message;

use std::net::UdpSocket;

fn main() {
    let message = message::Message {
        header: message::Header::new(),
        query: message::Query {},
    };

    let header = message::Header::create(
        1000, 0b0, 0b0101, 0b0, 0b0, 0b0, 0b0, 0b000, 0b0101, 1, 0, 0, 0,
    );

    println!(
        "QR: {}, OPCODE: {}, AA: {}, TC: {}, RD: {}, RA: {}, Z: {}, RCODE: {}",
        header.qr(),
        header.opcode(),
        header.aa(),
        header.tc(),
        header.rd(),
        header.ra(),
        header.z(),
        header.rcode()
    );

    println!("Hello, world!");

    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Couldn't bind to address");
    let buffer = header.to_byte();
    // let buffer = [
    //     0x3c, 0x0b, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x77, 0x77,
    //     0x77, 0x08, 0x6e, 0x79, 0x61, 0x6d, 0x69, 0x6b, 0x61, 0x6e, 0x03, 0x6e, 0x65, 0x74, 0x00,
    //     0x00, 0x01, 0x00, 0x01,
    // ];
    socket
        .send_to(&buffer, "192.168.12.1:53")
        .expect("Couldn't send");
    let mut buf = [0; 512];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!(
        "number_of_bytes: {}, src_addr: {:?}",
        number_of_bytes, src_addr
    );
    println!("{:?}", buf);
}
