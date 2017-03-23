#![allow(dead_code)]

use std::u64::{MAX as u64MAX};
use helper::NetHelper;

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
            let (converted, n) = NetHelper::bytes_to_u64(&Vec::from(buffer), pos);
            if !converted {
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
            if NetHelper::u64_to_bytes(self.parts[i], &mut ret_val, pos) == 0 {
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
    pub fn dividable(&self, number: u64) -> bool {
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

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        !(self.parts.len() > 0 && self.parts[0] != 0)
    }
}
