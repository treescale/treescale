extern crate mio;
extern crate rand;

use mio::{ Token };
use self::rand::Rng;
use std::collections::HashMap;

pub fn random_token() -> Token {
    let mut rng = rand::thread_rng();
    Token(rng.gen_range(10..1000000))
}

pub fn get_random_token_from_map<T>(container: &HashMap<Token, T>) -> Token {
    let mut rand_token = random_token();
    loop {
        if !container.contains_key(&rand_token) {
            return rand_token
        }
        rand_token = random_token();
    }
}
