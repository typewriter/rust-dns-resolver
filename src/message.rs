use packed_struct::prelude::*;

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

#[derive(PackedStruct)]
#[packed_struct(bit_numbering = "msb0", endian = "msb")]
pub struct Header {
    #[packed_field]
    pub id: u16,
    #[packed_field]
    pub flags: u16,
    #[packed_field]
    pub qd_count: u16,
    #[packed_field]
    pub an_count: u16,
    #[packed_field]
    pub ns_count: u16,
    #[packed_field]
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

    // packed_struct を使えば簡単だが、あえて
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

    pub fn parse(bytes: &[u8; 12]) -> Self {
        let unpacked = Header::unpack(bytes).expect("Unpack error");
        unpacked
    }
}

#[derive(Debug)]
pub struct Question {
    pub qname: Vec<u8>,
    pub qname_dec: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl Question {
    pub fn new(fqdn: &str, qtype: u16, qclass: u16) -> Self {
        let mut qname = Vec::new();
        for word in fqdn.split('.') {
            qname.push(word.len() as u8);
            qname.extend(word.as_bytes());
        }
        qname.push(0 as u8);

        Self {
            qname: qname,
            qname_dec: "".to_string(),
            qtype: qtype, // 1: A, 5: CNAME, 28: AAAA
            qclass: qclass,
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

    pub fn parse(resources: &[u8], count: usize) -> (Vec<Self>, usize) {
        let mut selfs = Vec::new();

        let mut position = 0;
        loop {
            // NAME
            let name_pair = Resource::extract_name(resources, resources, position);
            let name = name_pair.0;
            // let mut name_vec = Vec::new();
            // name_vec.extend(name.into_bytes());
            position = name_pair.1;
            // TYPE
            let qtype = u16::from(resources[position]) * 256 + u16::from(resources[position + 1]);
            position += 2;
            // CLASS
            let class = u16::from(resources[position]) * 256 + u16::from(resources[position + 1]);
            position += 2;

            let resource = Self {
                qname: Vec::new(),
                qname_dec: name,
                qtype: qtype,
                qclass: class,
            };
            selfs.push(resource);

            if selfs.len() >= count {
                break;
            }
        }

        (selfs, position)
    }
}

#[derive(Debug)]
pub enum IpAddr {
    V4(String),
    V6(String),
}

#[derive(Debug)]
pub struct Resource {
    pub name: String,
    pub rr_type: u16,
    pub data_class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,

    pub address: IpAddr,  // A, AAAA
    pub cname: String,    // CNAME
    pub nsdname: String,  // NS
    pub preference: u16,  // MX
    pub exchange: String, // MX
    pub mname: String,    // SOA
    pub rname: String,    // SOA
    pub serial: u32,      // SOA
    pub refresh: u32,     // SOA
    pub retry: u32,       // SOA
    pub expire: u32,      // SOA
    pub minimum: u32,     // SOA
    pub txt_data: String, // TXT
}

impl Resource {
    /** Message: メッセージ圧縮での参照に必要 */
    pub fn parse(message: &[u8], resources: &[u8], count: usize) -> Vec<Self> {
        let mut selfs = Vec::new();

        let mut position = 0;
        loop {
            // NAME
            let name_pair = Resource::extract_name(message, resources, position);
            let name = name_pair.0;
            position = name_pair.1;
            // TYPE
            let rr_type = u16::from(resources[position]) * 256 + u16::from(resources[position + 1]);
            position += 2;
            // CLASS
            let class = u16::from(resources[position]) * 256 + u16::from(resources[position + 1]);
            position += 2;
            // TTL
            let ttl = u32::from(resources[position]) * 256 * 256 * 256
                + u32::from(resources[position + 1]) * 256 * 256
                + u32::from(resources[position + 2]) * 256
                + u32::from(resources[position + 3]);
            position += 4;
            // RDLENGTH
            let rdlength =
                u16::from(resources[position]) * 256 + u16::from(resources[position + 1]);
            position += 2;
            // RDATA
            let mut rdata: Vec<u8> = Vec::new();
            let begin = position;
            let end = position + usize::from(rdlength);
            rdata.extend(resources[begin..end].into_iter());
            position += usize::from(rdlength);

            // タイプ別のフィールド
            let mut cname = "".to_string();
            if rr_type == 5 {
                let cname_tuple = Resource::extract_name(message, rdata.as_slice(), 0);
                cname = cname_tuple.0;
            }

            let mut nsdname = "".to_string();
            if rr_type == 2 {
                let nsdname_tuple = Resource::extract_name(message, rdata.as_slice(), 0);
                nsdname = nsdname_tuple.0;
            }

            let mut address = IpAddr::V4("".to_string());
            if rr_type == 1 && rdata.len() == 4 {
                address = IpAddr::V4(format!(
                    "{}.{}.{}.{}",
                    rdata[0], rdata[1], rdata[2], rdata[3]
                ));
            }
            if rr_type == 28 && rdata.len() == 16 {
                address = IpAddr::V6(format!(
                    "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                    u16::from(rdata[0]) * 256 + u16::from(rdata[1]),
                    u16::from(rdata[2]) * 256 + u16::from(rdata[3]),
                    u16::from(rdata[4]) * 256 + u16::from(rdata[5]),
                    u16::from(rdata[6]) * 256 + u16::from(rdata[7]),
                    u16::from(rdata[8]) * 256 + u16::from(rdata[9]),
                    u16::from(rdata[10]) * 256 + u16::from(rdata[11]),
                    u16::from(rdata[12]) * 256 + u16::from(rdata[13]),
                    u16::from(rdata[14]) * 256 + u16::from(rdata[15]),
                ));
            }

            let mut preference: u16 = 0;
            let mut exchange = "".to_string();
            if rr_type == 15 {
                preference = u16::from(rdata[0]) * 256 + u16::from(rdata[1]);
                let exchange_tuple = Resource::extract_name(message, rdata.as_slice(), 2);
                exchange = exchange_tuple.0;
            }

            let mut mname = "".to_string();
            let mut rname = "".to_string();
            let mut serial: u32 = 0;
            let mut refresh: u32 = 0;
            let mut retry: u32 = 0;
            let mut expire: u32 = 0;
            let mut minimum: u32 = 0;
            if rr_type == 6 {
                let mname_tuple = Resource::extract_name(message, rdata.as_slice(), 0);
                mname = mname_tuple.0;
                let rname_tuple = Resource::extract_name(message, rdata.as_slice(), mname_tuple.1);
                rname = rname_tuple.0;
                let offset = rname_tuple.1;
                let mut v = (0..5).map(|i| {
                    u32::from(rdata[offset + i * 4 + 0]) * 256 * 256 * 256
                        + u32::from(rdata[offset + i * 4 + 1]) * 256 * 256
                        + u32::from(rdata[offset + i * 4 + 2]) * 256
                        + u32::from(rdata[offset + i * 4 + 3])
                });
                serial = v.next().unwrap();
                refresh = v.next().unwrap();
                retry = v.next().unwrap();
                expire = v.next().unwrap();
                minimum = v.next().unwrap();
            }

            let mut txt_data = "".to_string();
            if rr_type == 16 {
                let end = usize::from(rdata[0]) + 1;
                txt_data = String::from_utf8_lossy(&rdata[1..end]).into_owned();
            }

            let resource = Self {
                name: name,
                rr_type: rr_type,
                data_class: class,
                ttl: ttl,
                rdlength: rdlength,
                rdata: rdata,
                cname: cname,
                nsdname: nsdname,
                address: address,
                preference: preference,
                exchange: exchange,
                mname: mname,
                rname: rname,
                serial: serial,
                refresh: refresh,
                retry: retry,
                expire: expire,
                minimum: minimum,
                txt_data: txt_data,
            };
            selfs.push(resource);

            if selfs.len() >= count {
                break;
            }
        }

        selfs
    }

    /** メッセージ圧縮に対応した NAME の抽出 */
    fn extract_name(message: &[u8], resources: &[u8], offset: usize) -> (String, usize) {
        let mut position = usize::from(offset);
        let mut name = Vec::new();
        loop {
            let length = resources[position];
            position += 1;
            // 末尾である
            if length == 0 {
                break;
            }
            // 圧縮である
            if length & 0b11000000 == 0b11000000 {
                let offset =
                    usize::from(length & 0b00111111) * 256 + usize::from(resources[position]);
                let reference_name = Resource::extract_name(message, message, offset);
                name.push(reference_name.0);
                position += 1;
                break;
            }
            // 通常のデータ
            let begin = position;
            let end = begin + usize::from(length);
            let chars = &resources[begin..end];
            let str = String::from_utf8_lossy(chars).into_owned();
            name.push(str);
            position += usize::from(length);
        }

        (itertools::join(name, "."), position)
    }
}

#[cfg(test)]
mod tests {
    use super::{Header, Question};

    #[test]
    fn header_bytes() {
        let header = Header::create(255, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0);

        let actual = header.to_byte();
        let expect: [u8; 12] = [0, 255, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0];
        assert_eq!(expect, actual);
    }

    #[test]
    fn header_parse() {
        let header: [u8; 12] = [
            0xF7, 0x67, 0x81, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];

        let parsed_header = Header::parse(&header);
        assert_eq!(parsed_header.id, 0xF767);
        assert_eq!(parsed_header.qr(), 1);
        assert_eq!(parsed_header.opcode(), 0);
        assert_eq!(parsed_header.aa(), 0);
        assert_eq!(parsed_header.tc(), 0);
        assert_eq!(parsed_header.rd(), 1);
        assert_eq!(parsed_header.ra(), 0);
        assert_eq!(parsed_header.z(), 0);
        assert_eq!(parsed_header.rcode(), 0);
        assert_eq!(parsed_header.qd_count, 1);
        assert_eq!(parsed_header.an_count, 2);
        assert_eq!(parsed_header.ns_count, 0);
        assert_eq!(parsed_header.ar_count, 0);
    }

    #[test]
    fn question_bytes() {
        let question = Question::new("nyamikan.net", 2, 1);

        let actual = question.to_byte();
        let expect = vec![
            8, 0x6e, 0x79, 0x61, 0x6d, 0x69, 0x6b, 0x61, 0x6e, 3, 0x6e, 0x65, 0x74, 0, 0, 2, 0, 1,
        ];
        assert_eq!(expect, actual);
    }

    #[test]
    fn question_parse() {
        let question = vec![
            3, 0x77, 0x77, 0x77, 8, 0x6e, 0x79, 0x61, 0x6d, 0x69, 0x6b, 0x61, 0x6e, 3, 0x6e, 0x65,
            0x74, 0, 0, 2, 0, 1,
        ];

        let parsed_questions = Question::parse(&question, 1);
        assert_eq!(parsed_questions.0.len(), 1);
        let parsed_question = &parsed_questions.0[0];
        assert_eq!(parsed_question.qname_dec, "www.nyamikan.net");
        assert_eq!(parsed_question.qtype, 2);
        assert_eq!(parsed_question.qclass, 1);
    }
}
