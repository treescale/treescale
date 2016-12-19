#![allow(dead_code)]
extern crate byteorder;

use std::io::{Result, Cursor, Error, ErrorKind};
use self::byteorder::{BigEndian, ReadBytesExt, ByteOrder};
use std::iter::FromIterator;

pub struct Event {
    pub path: String,
    pub name: String,
    pub from: String,
    pub target: String,
    pub public_data: String,
    pub data: Vec<u8>,
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
    pub fn from_raw(data: &Vec<u8>) -> Result<Event> {
        let mut offset = 0 as usize;
        let mut ev = Event::default();

        if data.len() <= 6 * 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Event data is too short to convert it!!"));
        }

        ev.path = match Event::read_field(&data, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.name = match Event::read_field(&data, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.from = match Event::read_field(&data, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.target = match Event::read_field(&data, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.public_data = match Event::read_field(&data, offset, false) {
            Ok((f, _, off)) => {
                offset = off;
                f
            }
            Err(e) => return Err(e)
        };

        ev.data = match Event::read_field(&data, offset, true) {
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
        BigEndian::write_u32(&mut len_buf, (data_len - 4) as u32);
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
    fn read_field(data: &Vec<u8>, off: usize, get_vec: bool) -> Result<(String, Vec<u8>, usize)> {
        let mut endian_bytes = vec![0; 4];
        let data_len = data.len() as usize;
        let mut offset = off as usize;
        for i in 0..4 {
            endian_bytes[i] = data[offset + i]
        }

        offset += 4;
        let mut rdr = Cursor::new(endian_bytes);
        let endian_len = rdr.read_u32::<BigEndian>().unwrap() as usize;
        if endian_len > (data_len - offset) {
            return Err(Error::new(ErrorKind::InvalidData, "error decoding given data"));
        }

        let d = Vec::from_iter(data[offset..offset + endian_len].iter().cloned());
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
        BigEndian::write_u32(len_buf, (filed_len) as u32);
        let mut off = offset;
        buf[off..off + 4].copy_from_slice(len_buf.as_slice());
        off += 4;
        buf[off..off + filed_len].copy_from_slice(data);
        Ok(())
    }
}
