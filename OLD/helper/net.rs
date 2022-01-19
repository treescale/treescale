#![allow(dead_code)]

use std::mem;

/// helper functions for network operations
pub struct NetHelper {
}

impl NetHelper {
    /// Converting u32 integer to BigEndian bytes
    /// Returns 0 if it is unable to make it
    /// Returns final offset in buffer after adding bytes to it
    #[inline(always)]
    pub fn u32_to_bytes(number: u32, buffer: &mut Vec<u8>, offset: usize) -> usize {
        if buffer.len() - offset < 4 {
            return 0;
        }

        let endian_bytes = unsafe {
            mem::transmute::<u32, [u8; 4]>(number.to_be())
        };

        buffer[offset + 0] = endian_bytes[0];
        buffer[offset + 1] = endian_bytes[1];
        buffer[offset + 2] = endian_bytes[2];
        buffer[offset + 3] = endian_bytes[3];

        // how many bytes we have written
        4
    }

    /// Converting u64 integer to BigEndian bytes
    /// Returns 0 if it is unable to make it
    /// Returns final offset in buffer after adding bytes to it
    #[inline(always)]
    pub fn u64_to_bytes(number: u64, buffer: &mut Vec<u8>, offset: usize) -> usize {
        if buffer.len() - offset < 8 {
            return 0;
        }

        let endian_bytes = unsafe {
            mem::transmute::<u64, [u8; 8]>(number.to_be())
        };

        buffer[offset + 0] = endian_bytes[0];
        buffer[offset + 1] = endian_bytes[1];
        buffer[offset + 2] = endian_bytes[2];
        buffer[offset + 3] = endian_bytes[3];
        buffer[offset + 4] = endian_bytes[4];
        buffer[offset + 5] = endian_bytes[5];
        buffer[offset + 6] = endian_bytes[6];
        buffer[offset + 7] = endian_bytes[7];

        // how many bytes we have written
        8
    }

    /// Parse given BigEndian bytes into u32 number
    #[inline(always)]
    pub fn bytes_to_u32(buffer: &Vec<u8>, offset: usize) -> (bool, u32) {
        if buffer.len() + offset < 4 {
            return (false, 0);
        }

        (true, unsafe {
            let a = [
                buffer[offset + 0], buffer[offset + 1],
                buffer[offset + 2], buffer[offset + 3],
            ];
            u32::from_be(mem::transmute::<[u8; 4], u32>(a))
        })
    }

    /// Parse given BigEndian bytes into u64 number
    #[inline(always)]
    pub fn bytes_to_u64(buffer: &Vec<u8>, offset: usize) -> (bool, u64) {
        if buffer.len() + offset < 8 {
            return (false, 0);
        }

        (true, unsafe {
            let a = [
                buffer[offset + 0], buffer[offset + 1],
                buffer[offset + 2], buffer[offset + 3],
                buffer[offset + 4], buffer[offset + 5],
                buffer[offset + 6], buffer[offset + 7],
            ];
            u64::from_be(mem::transmute::<[u8; 8], u64>(a))
        })
    }

    /// Checking if given Node value is valid or not
    /// Which means we will check it is Prime Number or not
    pub fn validate_value(value: u64) -> bool {
        match value {
            0 => true,
            1 => false,
            2 => true,
            3 => true,
            _ => {
                for i in 2..(value/2) {
                    if value % i == 0 {
                        return false
                    }
                }

                true
            }
        }
    }
}