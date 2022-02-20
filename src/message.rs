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
pub struct Resource {
    pub name: String,
    pub rr_type: u16,
    pub data_class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,

    pub cname: String,
    pub nsdname: String,
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

            let resource = Self {
                name: name,
                rr_type: rr_type,
                data_class: class,
                ttl: ttl,
                rdlength: rdlength,
                rdata: rdata,
                cname: cname,
                nsdname: nsdname,
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
