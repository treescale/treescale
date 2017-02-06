#![allow(dead_code)]
use std::mem;
use std::u32::MAX as u32MAX;
use std::u64::MAX as u64MAX;

/// Parse BigEndian Number from given bytes
/// NOTE: we will get only first 4 bytes from buffer
#[inline(always)]
pub fn parse_number(buffer: &[u8]) -> u32 {
    if buffer.len() < 4 {
        return u32MAX;
    }

    unsafe {
        let a = [
            buffer[0], buffer[1],
            buffer[2], buffer[3],
        ];
        u32::from_be(mem::transmute::<[u8; 4], u32>(a))
    }
}

/// Converting given number to BigEndian Bytes
#[inline(always)]
pub fn encode_number(buffer:&mut [u8], number: u32) -> bool {
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

    true
}

#[inline(always)]
pub fn parse_number64(buffer: &[u8]) -> u64 {
    if buffer.len() < 8 {
        return u64MAX;
    }

    unsafe {
        let a = [
            buffer[0], buffer[1],
            buffer[2], buffer[3],
            buffer[4], buffer[5],
            buffer[6], buffer[7],
        ];
        u64::from_be(mem::transmute::<[u8; 8], u64>(a))
    }
}

#[inline(always)]
pub fn encode_number64(buffer:&mut [u8], number: u64) -> bool {
    if buffer.len() < 8 {
        return false;
    }

    let endian_bytes = unsafe {
        mem::transmute::<u64, [u8; 8]>(number.to_be())
    };

    buffer[0] = endian_bytes[0];
    buffer[1] = endian_bytes[1];
    buffer[2] = endian_bytes[2];
    buffer[3] = endian_bytes[3];
    buffer[4] = endian_bytes[4];
    buffer[5] = endian_bytes[5];
    buffer[6] = endian_bytes[6];
    buffer[7] = endian_bytes[7];

    true
}

/// Base struct for handling path information and processing it
pub struct Path {
    // parts for path calculations
    parts: Vec<u64>
}

impl Path {
    #[inline(always)]
    pub fn new() -> Path {
        Path {
            parts: vec![]
        }
    }

    #[inline(always)]
    pub fn from_bytes(buffer: &[u8]) -> Option<Path> {
        if buffer.len() % 8 != 0 {
            return None;
        }

        let mut p = Path::new();
        for i in 0..(buffer.len() / 8) {
            let pos = i * 8;
            let n = parse_number64(&buffer[pos..pos+8]);
            if n == u64MAX {
                return None;
            }
            p.parts.push(n);
        }

        Some(p)
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        let part_count = self.parts.len();
        let path_len = part_count * 8;
        let mut ret_val = vec![0u8; path_len];
        for i in 0..part_count {
            let pos = i * 8;
            if !encode_number64(&mut ret_val[pos..pos+8], self.parts[i]) {
                return None;
            }
        }

        Some(ret_val)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.parts.len() * 8
    }

    #[inline(always)]
    pub fn mul(&mut self, number: u64) {
        // we can't have 0 as a path multiplication
        if number == 0 {
            return;
        }

        // if after multiplication our number will overflow u64
        // just adding new slot and keeping value there
        if self.parts.len() == 0 {
            self.parts.push(number);
            return;
        }

        let last_index = self.parts.len() - 1;
        if self.parts[last_index] > u64MAX / number {
            self.parts.push(number);
            return;
        }

        // otherwise just making usual multiplication
        self.parts[last_index] *= number;
    }

    #[inline(always)]
    pub fn div(&mut self, number: u64) -> bool {
        if number == 0 {
            return false;
        }

        for i in 0..self.parts.len() {
            if self.parts[i] % number == 0 {
                self.parts[i] /= number;
                return true;
            }
        }

        false
    }

    #[inline(always)]
    pub fn dividable(&mut self, number: u64) -> bool {
        if number == 0 {
            return false;
        }

        for i in 0..self.parts.len() {
            if self.parts[i] % number == 0 {
                return true;
            }
        }

        false
    }
}
