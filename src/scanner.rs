extern crate config;
extern crate base64;

use std::time;
use std::thread;
use std::process::{Command, Child};
use std::io::Write;
use std::io::Read;


pub struct Scanner {
    pub name: String,
    pub path: String,
    pub config: config::Config,
    pub process: Command,
    pub pipe: Child
}


impl Scanner {
    /// Send message to stdin of process
    fn send(&mut self, message : &[u8]) {
        // encode to string
        let string_message: String = String::from_utf8(message.to_vec()).unwrap();

        // encode to base64
        let base64_message = base64::encode(&string_message.to_string()) + "\n";

        let stdin = self.pipe.stdin.as_mut().unwrap();
        stdin.write_all(&base64_message.as_bytes()).unwrap();
    }

    /// Receive message from stdout of process
    fn recv(&mut self) -> Result<Vec<u8>, String> {
        // ready buffer space
        let mut buffer = [0; 1024];

        // read 1024 byte
        match self.pipe.stdout.as_mut().unwrap().read(&mut buffer) {
            Ok(_) => {
                return Ok(buffer.to_vec());
            },
            Err(_err) => {
                return Err(
                    "[Err] Encode String Error\n\
                    Please check scanner's output correct.\n\n".to_string()
                );
            }
        };
    }

    /// Send request
    pub fn request(&mut self, message : &[u8]) -> Result<String, String> {
        // send message
        self.send(message);

        // ready response space
        let mut response = String::from("");

        loop {
            // receive message
            let buffer : Vec<u8> = match self.recv() {
                Ok(_buffer) => _buffer,
                Err(_err) => {
                    return Err(_err);
                }
            };

            // encode from u8 vector to string
            let buffer = match String::from_utf8(buffer) {
                Ok(_buffer) => _buffer,
                Err(_err) => {
                    return Err(
                        "[Err] Encode String Error\n\
                        Please check scanner's output correct.\n\n".to_string()                    )
                }
            };

            // if not contain '\n',
            // add buffer to more received
            if !buffer.contains('\n') {
                response += &buffer;
                let sleep_time = time::Duration::from_millis(10);
                thread::sleep(sleep_time);
                continue;
            }
            
            // split string at '\n'
            let splited: Vec<&str> = buffer.split('\n').collect();
            response += splited.get(0).unwrap();

            // decode base64
            match base64::decode(&response) {
                Ok(_decode) => {
                    match String::from_utf8(_decode) {
                        Ok(_decode) => {
                            return Ok(_decode);
                        },
                        Err(_err) => {
                            return Err(
                                "[Err] Decode Base64 Error\n\
                                Please check scanner's output correct.\n\n".to_string()
                            );
                        }
                    };
                },
                Err(_err) => {
                    return Err(
                        "[Err] Decode Base64 Error\n\
                        Please check scanner's output correct.\n\n".to_string()
                    );
                }
            };
        }
    }
}