extern crate chrono;

use self::chrono::prelude::UTC;

pub struct Log {
}

impl Log {
    #[inline(always)]
    fn print(log_type: &str, message: &str) {
        println!("[{}] [{}] -> {}",
                 UTC::now().to_rfc3339(),
                 log_type,
                 message);
    }

    #[inline(always)]
    pub fn Error(message: &str) {
        Log::print("ERROR", message);
    }

    #[inline(always)]
    pub fn Info(message: &str) {
        Log::print("INFO", message);
    }

    #[inline(always)]
    pub fn Warn(message: &str) {
        Log::print("WARNING", message);
    }
}