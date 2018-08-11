extern crate config;
extern crate base64;

use std::time;
use std::thread;
use std::process::{Command, Child, Stdio};
use std::io::{Read, Write};
use config::Config;


pub struct Scanner {
    pub name: String,
    pub path: String,
    pub config: config::Config,
    pub process: Command,
    pub pipe: Child,
    pub work: bool
}


impl Scanner {
    /// Build new scanner
    pub fn new(name: String, config:Config, path:String, executable:String) -> Scanner{
        let mut process = Command::new(executable);
        let pipe = process
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().unwrap();
        
        Scanner {
            name: name,
            path: path,
            config: config,
            process: process,
            pipe: pipe,
            work: true
        }
    }

    /// Send message to stdin of process
    fn send(&mut self, message : &[u8]) -> Result<(), String>{
        let stdin = self.pipe.stdin.as_mut().unwrap();

        match stdin.write_all(message) {
            Ok(_) => {
                return Ok(())
            },
            Err(_err) => {
                return Err(
                    "[Unexpected Err] Send Message Error\n\
                    Please check scanner's input correct.\n\n".to_string()
                )
            }
        };
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
                    "[Unexpected Err] Encode String Error\n\
                    Please check scanner's output correct.\n\n".to_string()
                );
            }
        };
    }

    /// Wait and receive line of string message
    fn recv_line(&mut self) -> Result<String, String> {
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
                        "[Unexpected Err] Encode String Error\n\
                        Please check scanner's output correct.\n\n".to_string()
                    )
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

            return Ok(response);
        }
    }

    /// Send request (message is base64 string)
    pub fn request(&mut self, message: String) -> Result<String, String> {
        let message = message + "\n";

        // send message
        match self.send(message.as_bytes()) {
            Ok(_) => {},
            Err(_err) => {
                return Err(_err);
            }
        }

        // ready response space
        let response = match self.recv_line(){
            Ok(_line) => _line,
            Err(_err) => {
                return Err(_err);
            }
        };

        // decode base64
        match base64::decode(&response) {
            Ok(_decode) => {
                match String::from_utf8(_decode) {
                    Ok(_decode) => {
                        return Ok(_decode);
                    },
                    Err(_err) => {
                        return Err(
                            "[Unexpected Err] Decode Base64 Error\n\
                            Please check scanner's output correct.\n\n".to_string()
                        );
                    }
                };
            },
            Err(_err) => {
                return Err(
                    "[Unexpected Err] Decode Base64 Error\n\
                    Please check scanner's output correct.\n\n".to_string()
                );
            }
        };
    }

    /// Send request (message is bytes and encode base64 here)
    pub fn request_by_bytes(&mut self, message: &[u8]) -> Result<String, String> {
        if !self.work {
            return Ok("".to_string());
        }

        // encode to base64
        let string_message: String = String::from_utf8(message.to_vec()).unwrap();
        let base64_message = base64::encode(&string_message.to_string());

        self.request(base64_message)
    }

    /// Send empty line and received result
    pub fn request_end(&mut self) -> Result<String, String> {
        let result = self.request("\n".to_string());
        self.kill();
        return result;
    }

    /// Kill the process
    /// make work-flag false
    pub fn kill(&mut self) {
        self.work = false;

        match self.pipe.kill() {
            Ok(_) => {},
            Err(_err) => {
                println!("[Unexpected Err] Drop Scanner Error\n\
                    Please check is scanner procces correct.\n\n\
                    {}", _err);
            }
        }
    }
}


impl Drop for Scanner {
    /// when scope out, kill process
    fn drop(&mut self) {
        self.work = false;

        match self.pipe.kill() {
            Ok(_) => {},
            Err(_err) => {
                println!("[Unexpected Err] Drop Scanner Error\n\
                    Please check is scanner procces correct.\n\n\
                    {}", _err);
            }
        }
    }
}