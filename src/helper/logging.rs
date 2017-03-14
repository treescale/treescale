#![allow(dead_code)]
extern crate chrono;

use self::chrono::prelude::UTC;

pub struct Log {
}

impl Log {
    #[inline(always)]
    fn print(log_type: &str, message: &str, err: &str) {
        println!("[{}] [{}] - {} -> {}",
                 UTC::now().to_rfc3339(),
                 log_type,
                 message,
                 err);
    }

    #[inline(always)]
    pub fn error(message: &str, err: &str) {
        Log::print("ERROR", message, err);
    }

    #[inline(always)]
    pub fn info(message: &str, err: &str) {
        Log::print("INFO", message, err);
    }

    #[inline(always)]
    pub fn warn(message: &str, err: &str) {
        Log::print("WARNING", message, err);
    }
}
