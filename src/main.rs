mod message;

use std::net::UdpSocket;

fn main() {
    let message = message::Message {
        header: message::Header::create(
            1000, 0b0, 0b0000, 0b0, 0b0, 0b1, 0b0, 0b000, 0b0000, 1, 0, 0, 0,
        ),
        question: message::Question::new("www.nyamikan.net"),
    };

    // pack -> unpack test
    let header_bytes = message.header.to_byte();
    let header_unpacked = message::Header::parse(&header_bytes);

    print_header(&message.header);
    print_header(&header_unpacked);

    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Couldn't bind to address");
    let buffer = message.to_bytes();
    let buffer_slice = buffer.as_slice();
    socket
        .send_to(&buffer_slice, "192.168.12.1:53")
        .expect("Couldn't send");
    let mut buf = [0; 512];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!(
        "number_of_bytes: {}, src_addr: {:?}",
        number_of_bytes, src_addr
    );
    println!("{:?}", buf);
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
