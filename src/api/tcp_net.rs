#![allow(dead_code)]
extern crate mio;

use self::mio::{Token, Ready};

use api::NodeApi;
use helper::conn::Connection;
use helper::NetHelper;
use event::Event;

pub trait ApiTcpNetwork {
    /// Handling TCP event, if it is used to be for TCP handler
    /// function will return "true", otherwise "false"
    fn tcp_ready(&mut self, token: Token, kind: Ready) -> bool;

    /// Handling Readable event from TCP connection
    fn tcp_readable(&mut self, token: Token);

    /// Handling Writable event from TCP connection
    fn tcp_writable(&mut self, token: Token);

    /// Closing connection from TCP side
    fn tcp_close(&mut self, token: Token);

    /// Check handshake information, and try to read it, if not exists yet
    fn tcp_handshake(&mut self, token: Token) -> bool;
}


impl ApiTcpNetwork for NodeApi {
    #[inline]
    fn tcp_ready(&mut self, token: Token, kind: Ready) -> bool {
        // if this event is not TCP event just returning false
        // for letting know that TCP not allowed to handle this event
        if !self.tcp_connections.contains(token) {
            return false;
        }

        // if we have error in event, then we need to close it
        if kind.is_error() || kind.is_hup() {

            self.tcp_close(token);

        } else if kind.is_readable() {

            self.tcp_readable(token);

        } else if kind.is_writable() {

            self.tcp_writable(token);

        }

        true
    }

    #[inline]
    fn tcp_readable(&mut self, token: Token) {
        // if we don't have handshake information
        // or it's not validated, just returning
        if !self.tcp_handshake(token) {
            return;
        }
        let (close_conn, mut data) = {
            let ref mut conn = self.tcp_connections[token];
            match conn.read_data() {
                Some(d) => (false, d),
                None => (true, vec![])
            }
        };

        if close_conn {
            self.tcp_close(token);
            return;
        }

        while !data.is_empty() {
            let buffer = data.remove(0);
            self.handle_event(match Event::from_raw(&buffer) {
                Some(e) => e,
                None => continue
            });
        }
    }

    #[inline]
    fn tcp_writable(&mut self, token: Token) {
        // if we got here then we have connection with this token
        let close_conn = {
            let ref mut conn = self.tcp_connections[token];
            match conn.flush() {
                Some(done) => {
                    // if Write queue is not empty, just returning
                    // and waiting for the next cycle
                    if !done {
                        return;
                    }

                    // making connection readable because we don't have anything to write
                    conn.make_readable(&self.poll);

                    // letting know to keep connection
                    // so that we can make sure that queue is empty
                    false
                }

                // closing connection if we have write error
                None => true
            }
        };

        if close_conn {
            self.tcp_close(token);
        }
    }

    #[inline]
    fn tcp_close(&mut self, token: Token) {
        // if we got here then most probably we have this connection in the list
        let conn = match self.tcp_connections.remove(token) {
            Some(c) => c,
            None => return
        };

        // shutting down socket
        conn.close();

        // removing current identity from connection
        self.remove_identity(conn.conn_token.clone(), token);
    }

    #[inline]
    fn tcp_handshake(&mut self, token: Token) -> bool {
        // if we got here then we have connection with this token
        let mut close_conn = {
            let ref mut conn = self.tcp_connections[token];
            // if we don't have yet API version defined
            if !Connection::check_api_version(conn.api_version) {
                match conn.read_api_version() {
                    Some((done, version)) => {
                        // if we not done with reading API version
                        // Just returning and waiting until next readable cycle
                        if !done {
                            return false;
                        }

                        // if we got wrong API version just closing connection
                        if !Connection::check_api_version(version) {
                            true
                        } else {
                            // if we got valid API version
                            // saving it as a connection version
                            conn.api_version = version;
                            false
                        }
                    }

                    // if we have connection error closing it
                    None => true
                }
            } else {
                false
            }
        };

        if close_conn {
            self.tcp_close(token);
            return false;
        }

        close_conn = {
            let ref mut conn = self.tcp_connections[token];
            // if we don't have connection token, then we need to read it
            if conn.conn_token.len() == 0 {
                // reading Connection Token and Value
                match conn.read_token_value() {
                    Some((done, token_str, value)) => {
                        // if we not done with reading API version
                        // Just returning and waiting until next readable cycle
                        if !done {
                            return false;
                        }

                        // checking if we got valid Prime Value or not
                        // if it's invalid just closing connection
                        if !NetHelper::validate_value(value) {
                            true
                        } else {
                            // if we done with token and value
                            // just setting them for connection
                            // and writing API handshake information
                            conn.conn_token = token_str;
                            conn.conn_value = value;

                            false
                        }
                    }

                    // if we have connection error closing it
                    None => true
                }
            } else {
                false
            }
        };

        if close_conn {
            self.tcp_close(token);
            return false;
        }

        // if we got here then we have API version, Token and value
        true
    }
}