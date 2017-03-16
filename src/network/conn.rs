#![allow(dead_code)]

use node::MAX_API_VERSION;

pub struct Connection {

}

impl Connection {
    /// Checking API version, if it's not correct function will return false
    #[inline(always)]
    pub fn check_api_version(version: u32) -> bool {
        version > 0 && version < MAX_API_VERSION
    }
}