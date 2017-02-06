#![allow(dead_code)]
use std::io::{Result, Error, ErrorKind};
use std::mem;
use std::u32::MAX as u32MAX;

pub struct Event {
    pub path: String,
    pub name: String,
    pub from: String,
    pub target: String,
    pub public_data: String,
    pub data: Vec<u8>,
}

/// Parse BigEndian Number from given bytes
/// NOTE: we will get only first 4 bytes from buffer
#[inline(always)]
fn parse_number(buffer: &[u8]) -> u32 {
    if buffer.len() < 4 {
        return u32MAX;
    }

    return unsafe {
        let a = [
            buffer[0], buffer[1],
            buffer[2], buffer[3],
        ];
        u32::from_be(mem::transmute::<[u8; 4], u32>(a))
    };
}

/// Converting given number to BigEndian Bytes
#[inline(always)]
fn encode_number(buffer:&mut [u8], number: u32) -> bool {
    if buffer.len() < 4 {
        return false;
    }

    let endian_bytes = unsafe {
        mem::transmute::<u32, [u8; 4]>(number.to_be())
    };

    buffer[0] = endian_bytes[0];
    buffer[1] = endian_bytes[1];
    buffer[2] = endian_bytes[2];
    buffer[3] = endian_bytes[3];

    return true;
}

impl Event {
    #[inline(always)]
    pub fn default() -> Event {
        Event {
            path: String::new(),
            name: String::new(),
            from: String::new(),
            target: String::new(),
            public_data: String::new(),
            data: vec![],
        }
    }

    #[inline(always)]
    pub fn from_raw(data: &Vec<u8>) -> Result<Event> {
        let mut offset = 0 as usize;
        let mut ev = Event::default();
        let mut endian_bytes = vec![0; 4];
        let data_len = data.len();

        if data.len() <= 6 * 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Event data is too short to convert it!!"));
        }

        ev.path = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.name = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.from = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.target = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.public_data = match Event::read_field(&data, &mut endian_bytes, data_len, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.data = match Event::read_field(&data, &mut endian_bytes, data_len, offset, true) {
            Ok((_, f, _)) => {
                f
            }
            Err(e) => return Err(e)
        };

        Ok(ev)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        let (path_len, name_len, from_len, target_len, public_data_len, event_data_len)
                    = (self.path.len(), self.name.len(), self.from.len(), self.target.len(), self.public_data.len(), self.data.len());

        // calculating total data length
        let data_len = 4
            + path_len + 4
            + name_len + 4
            + from_len + 4
            + target_len + 4
            + public_data_len + 4
            + event_data_len + 4;

        let mut buf: Vec<u8> = vec![0; data_len];
        let mut len_buf: Vec<u8> = vec![0; 4];
        let mut offset = 0;

        // writing full data length only
        encode_number(&mut len_buf, (data_len - 4) as u32);
        buf[0..4].copy_from_slice(len_buf.as_slice());
        offset += 4;

        // setting path data here
        match Event::write_field(&mut len_buf, &mut buf, self.path.as_bytes(), path_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + path_len;

        // setting name data here
        match Event::write_field(&mut len_buf, &mut buf, self.name.as_bytes(), name_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + name_len;

        // setting "from" data here
        match Event::write_field(&mut len_buf, &mut buf, self.from.as_bytes(), from_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + from_len;

        // setting target data here
        match Event::write_field(&mut len_buf, &mut buf, self.target.as_bytes(), target_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + target_len;

        // setting public_data data here
        match Event::write_field(&mut len_buf, &mut buf, self.public_data.as_bytes(), public_data_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        offset += 4 + public_data_len;

        // setting "data" data here
        match Event::write_field(&mut len_buf, &mut buf, self.data.as_slice(), event_data_len, offset) {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
        // offset += 4 + data_len;

        Ok(buf)
    }

    #[inline(always)]
    fn read_field(data: &Vec<u8>, endian_bytes: &mut Vec<u8>, data_len: usize, off: usize, get_vec: bool) -> Result<(String, Vec<u8>, usize)> {
        let mut offset = off as usize;
        for i in 0..4 {
            endian_bytes[i] = data[offset + i]
        }

        offset += 4;
        let endian_len = parse_number(endian_bytes.as_slice()) as usize;
        if endian_len > (data_len - offset) {
            return Err(Error::new(ErrorKind::InvalidData, "error decoding given data"));
        }

        let d = Vec::from(&data[offset..offset + endian_len]);
        if get_vec {
            return Ok((String::new(), d, offset + endian_len));
        }

        Ok(match String::from_utf8(d) {
            Ok(s) => (s, vec![], offset + endian_len),
            Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Unable to convert data to string"))
        })
    }

    #[inline(always)]
    fn write_field(len_buf: &mut Vec<u8>, buf: &mut Vec<u8>, data: &[u8], filed_len: usize, offset: usize) -> Result<()> {
        // Writing Path
        encode_number(len_buf, (filed_len) as u32);
        let mut off = offset;
        buf[off..off + 4].copy_from_slice(len_buf.as_slice());
        off += 4;
        buf[off..off + filed_len].copy_from_slice(data);
        Ok(())
    }
}
