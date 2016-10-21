
use error::codes::ErrorCodes;

pub struct Error {

}

impl Error {
    pub fn handle_error(code: ErrorCodes, message: &str, from: &str) {
        println!("Error from {:?} -> {:?}", from, message);
    }
}
