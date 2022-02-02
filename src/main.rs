mod message;

use std::net::UdpSocket;

use crate::message::Header;

fn main() {
    // Request
    let message = message::Message {
        header: message::Header::create(
            // 1234, 0b1, 0b0101, 0b0, 0b1, 0b1, 0b1, 0b010, 0b1110, 65531, 255, 54,
            // 2091,
            1000, 0b0, 0b0000, 0b0, 0b0, 0b1, 0b0, 0b000, 0b0000, 1, 0, 0, 0,
        ),
        question: message::Question::new("www.nyamikan.net"),
    };
    print_header(&message.header);

    // Send
    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Couldn't bind to address");
    let buffer = message.to_bytes();
    let buffer_slice = buffer.as_slice();
    socket
        .send_to(&buffer_slice, "192.168.12.1:53")
        .expect("Couldn't send");

    // Receive
    let mut buf = [0; 512];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!(
        "number_of_bytes: {}, src_addr: {:?}",
        number_of_bytes, src_addr
    );
    println!("{:?}", buf);

    // Response
    let mut ret_header: [u8; 12] = Default::default();
    ret_header.copy_from_slice(&buf[0..12]);
    let ret_header = message::Header::parse(&ret_header);
    print_header(&ret_header);

    let ret_body = &buf[12..512];
    let questions = message::Question::parse(&ret_body, ret_header.qd_count.into());
    println!("{:?}", questions.0);

    let ret_body = &buf[(questions.1 + 12)..512];
    let resources = message::Resource::parse(
        &buf,
        ret_body,
        (ret_header.an_count + ret_header.ns_count + ret_header.ar_count).into(),
    );

    println!("{:?}", resources);
}

fn print_header(header: &message::Header) {
    println!(
        "ID: {}, QR: {}, OPCODE: {}, AA: {}, TC: {}, RD: {}, RA: {}, Z: {}, RCODE: {}, QDCOUNT: {}, ANCOUNT: {}, NSCOUNT: {}, ARCOUNT: {}",
        header.id,
        header.qr(),
        header.opcode(),
        header.aa(),
        header.tc(),
        header.rd(),
        header.ra(),
        header.z(),
        header.rcode(),
        header.qd_count,
        header.an_count,
        header.ns_count,
        header.ar_count,
    );
}
