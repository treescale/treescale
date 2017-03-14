#![allow(dead_code)]

use helper::{Path, NetHelper, Log};
use std::error::Error;

pub struct Event {
    pub path: Path,
    pub name: String,
    pub from: String,
    pub target: String,
    pub data: Vec<u8>,
}

impl Event {
    #[inline(always)]
    pub fn default() -> Event {
        Event {
            path: Path::new(),
            name: String::new(),
            from: String::new(),
            target: String::new(),
            data: vec![],
        }
    }

    #[inline(always)]
    pub fn from_raw(data: &Vec<u8>) -> Option<Event> {
        let mut offset: usize = 0;
        let mut ev = Event::default();
        let data_len = data.len();

        // Reading Path Field from data
        ev.path = match Event::read_field(&data, offset, data_len) {
            Some((field_data, field_len)) => {
                offset += field_len;
                match Path::from_bytes(field_data) {
                    Some(p) => p,
                    None => Path::new()
                }
            }

            None => {
                Log::warn("Unable to Parse Path from Event Message", "Error while trying to read Path Filed");
                return None;
            }
        };

        // Reading Event Name
        ev.name = match Event::read_field(&data, offset, data_len) {
            Some((field_data, field_len)) => {
                offset += field_len;
                match String::from_utf8(Vec::from(field_data)) {
                    Ok(s) => s,
                    Err(e) => {
                        Log::warn("Unable to parse Event Name from raw data", e.description());
                        return None;
                    }
                }
            }

            None => {
                Log::warn("Unable to Parse Name from Event Message", "Error while trying to read Path Filed");
                return None;
            }
        };

        // Reading Event From
        ev.from = match Event::read_field(&data, offset, data_len) {
            Some((field_data, field_len)) => {
                offset += field_len;
                match String::from_utf8(Vec::from(field_data)) {
                    Ok(s) => s,
                    Err(e) => {
                        Log::warn("Unable to parse Event From field from raw data", e.description());
                        return None;
                    }
                }
            }

            None => {
                Log::warn("Unable to Parse From field from Event Message", "Error while trying to read Path Filed");
                return None;
            }
        };

        // Reading Event From
        ev.target = match Event::read_field(&data, offset, data_len) {
            Some((field_data, field_len)) => {
                offset += field_len;
                match String::from_utf8(Vec::from(field_data)) {
                    Ok(s) => s,
                    Err(e) => {
                        Log::warn("Unable to parse Event Target field from raw data", e.description());
                        return None;
                    }
                }
            }

            None => {
                Log::warn("Unable to Parse Target field from Event Message", "Error while trying to read Path Filed");
                return None;
            }
        };

        // we got all fields in event
        // so remaining data is for event data field
        ev.data = Vec::from(&data[offset..]);
        Some(ev)
    }

    #[inline(always)]
    fn read_field(data: &Vec<u8>, offset: usize, data_len: usize) -> Option<(&[u8], usize)> {
        let (converted, filed_len) = NetHelper::bytes_to_u32(&data, offset);
        let filed_len = filed_len as usize;
        if !converted || offset + filed_len >= data_len {
            return None
        }

        Some((&data[offset..(offset + filed_len)], filed_len))
    }

    #[inline(always)]
    pub fn to_raw(&self) -> Option<Vec<u8>> {
        let (path_len, name_len, from_len, target_len, event_data_len)
              = (self.path.len(), self.name.len(), self.from.len(), self.target.len(), self.data.len());

        let data_len = 4 + path_len // path len endian and path bytes len
            + 4 + name_len // name len endian and name bytes len
            + 4 + from_len // from len endian and from bytes len
            + 4 + target_len // target len endian and target bytes len
            + event_data_len; // event data bytes len

        // Adding +4 because we need to write also big endian total data length
        let mut buffer: Vec<u8> = vec![0; (data_len + 4)];
        let mut offset: usize = 0;

        // writing total data length
        offset += NetHelper::u32_to_bytes(data_len as u32, &mut buffer, offset);

        // writing Event Path field
        offset += NetHelper::u32_to_bytes(path_len as u32, &mut buffer, offset);
        match self.path.to_bytes() {
            Some(path_data) => {
                buffer[offset..offset + path_len].copy_from_slice(path_data.as_slice());
                offset += path_len;
            }

            None => {
                Log::warn("Unable to convert Event Path into Data bytes", "");
                return None;
            }
        }

        // Writing Event Name Field
        offset += NetHelper::u32_to_bytes(name_len as u32, &mut buffer, offset);
        buffer[offset..offset + name_len].copy_from_slice(self.name.as_bytes());
        offset += name_len;


        // Writing Event From Field
        offset += NetHelper::u32_to_bytes(from_len as u32, &mut buffer, offset);
        buffer[offset..offset + from_len].copy_from_slice(self.from.as_bytes());
        offset += from_len;


        // Writing Event From Field
        offset += NetHelper::u32_to_bytes(target_len as u32, &mut buffer, offset);
        buffer[offset..offset + target_len].copy_from_slice(self.target.as_bytes());
        offset += target_len;

        // remaining should be out event data
        buffer[offset..].copy_from_slice(self.data.as_slice());

        Some(buffer)
    }
}