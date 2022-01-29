pub struct Message {
    pub header: Header,
    pub question: Question,
}

impl Message {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend(self.header.to_byte());
        vec.extend(self.question.to_byte());
        vec
    }
}

pub struct Header {
    pub id: u16,
    pub flags: u16,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 1000,
            flags: 0,
            qd_count: 1,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        }
    }

    pub fn create(
        id: u16,
        qr: u8,
        opcode: u8,
        aa: u8,
        tc: u8,
        rd: u8,
        ra: u8,
        z: u8,
        rcode: u8,
        qd_count: u16,
        an_count: u16,
        ns_count: u16,
        ar_count: u16,
    ) -> Self {
        Self {
            id: id,
            flags: ((qr.wrapping_shl(7)
                + opcode.wrapping_shl(3)
                + aa.wrapping_shl(2)
                + tc.wrapping_shl(1)
                + rd) as u16)
                .wrapping_shl(8)
                + (ra.wrapping_shl(7) + z.wrapping_shl(4) + rcode) as u16,
            qd_count: qd_count,
            an_count: an_count,
            ns_count: ns_count,
            ar_count: ar_count,
        }
    }

    pub fn qr(&self) -> u8 {
        (self.flags >> 15 & 0b0001) as u8
    }

    pub fn opcode(&self) -> u8 {
        (self.flags >> 11 & 0b1111) as u8
    }

    pub fn aa(&self) -> u8 {
        (self.flags >> 10 & 0b0001) as u8
    }

    pub fn tc(&self) -> u8 {
        (self.flags >> 9 & 0b0001) as u8
    }

    pub fn rd(&self) -> u8 {
        (self.flags >> 8 & 0b0001) as u8
    }

    pub fn ra(&self) -> u8 {
        (self.flags >> 7 & 0b0001) as u8
    }

    pub fn z(&self) -> u8 {
        (self.flags >> 4 & 0b0111) as u8
    }

    pub fn rcode(&self) -> u8 {
        (self.flags & 0b1111) as u8
    }

    pub fn to_byte(&self) -> [u8; 12] {
        let mut bytes: [u8; 12] = [0; 12];
        bytes[0] = (self.id / 256) as u8;
        bytes[1] = (self.id % 256) as u8;
        bytes[2] = (self.flags / 256) as u8;
        bytes[3] = (self.flags % 256) as u8;
        bytes[4] = (self.qd_count / 256) as u8;
        bytes[5] = (self.qd_count % 256) as u8;
        bytes[6] = (self.an_count / 256) as u8;
        bytes[7] = (self.an_count % 256) as u8;
        bytes[8] = (self.ns_count / 256) as u8;
        bytes[9] = (self.ns_count % 256) as u8;
        bytes[10] = (self.ar_count / 256) as u8;
        bytes[11] = (self.ar_count % 256) as u8;
        bytes
    }
}

pub struct Question {
    pub qname: Vec<u8>,
    pub qtype: u16,
    pub qclass: u16,
}

impl Question {
    pub fn new(fqdn: &str) -> Self {
        let mut qname = Vec::new();
        for word in fqdn.split('.') {
            qname.push(word.len() as u8);
            qname.extend(word.as_bytes());
        }
        qname.push(0 as u8);

        Self {
            qname: qname,
            qtype: 1, // 1: A, 5: CNAME, 28: AAAA
            qclass: 1,
        }
    }

    pub fn to_byte(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.qname.iter());
        bytes.push((self.qtype / 256) as u8);
        bytes.push((self.qtype % 256) as u8);
        bytes.push((self.qclass / 256) as u8);
        bytes.push((self.qclass % 256) as u8);
        bytes
    }
}
