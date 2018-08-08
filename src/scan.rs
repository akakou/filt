extern crate base64;
extern crate config;

use std::vec::Vec;
use std::fs;
use std::env;
use std::time;
use std::thread;
use std::path::PathBuf;
use std::process::{Command, Stdio, Child};
use std::io::Write;
use std::io::Read;


struct Scanner {
    name: String,
    path: String,
    config: config::Config,
    process: Command,
    pipe: Child
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

    /// recieve message from stdout of process
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

    /// send request
    fn request(&mut self, message : &[u8]) -> Result<String, String> {
        // send message
        self.send(message);

        // ready response space
        let mut response = String::from("");

        loop {
            // recieve message
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
            // add buffer to more recieved
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


/// Search files which named `file_names` 
/// under `directory_path`.
fn search_files(files_paths: &mut Vec<PathBuf>, directory_path: PathBuf) {
    /* Get file names under directory_path */
    let paths = match fs::read_dir(directory_path) {
        Ok(_paths) => _paths,
        Err(_err) => {
            println!("[Unexpected Err] Reading Directory Unexpected Error\n\
                Language Error:{}\n", _err);
            return;
        }
    };
    
    /* Add files paths to vector */
    for unwraped_path in paths {
        let mut path = unwraped_path.unwrap().path();

        if path.is_dir() {
            // if `recursive == true`, search files recursive
            search_files(files_paths, path);
        } else {
            let _file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            // if file_name and argument file_name is same,
            // push file to vector
            files_paths.push(path);
        }
    }
}

/// Using signatures and scanner to scan target.
pub fn scan() {
    /* Get scanner */
    let mut scanners: Vec<Scanner> = Vec::new();

    // get current directory
    let unwraped_current_path = env::current_dir().unwrap();
    let current_path = unwraped_current_path.to_str().unwrap();

    // read config
    let mut settings = config::Config::default();
    match settings.merge(config::File::with_name("conf/Scanner")) {
        Ok(_) => {},
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Scanner.toml exists.\n\n\
                {}", _err);
            return;
        }
    }

    // get scanner names
    let scanners_names = match settings.get_array("scanners") {
        Ok(_scanners) => _scanners,
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Server.toml has `scanners` array.\n\n\
                {}", _err);
            return;
        }
    };

    // make scanner struct
    for unwraped_scanner_name in scanners_names {
        // get scanner path
        let mut scanner_name = match unwraped_scanner_name.into_str() {
            Ok(_scanner) =>  _scanner,
            Err(_err) => {
                println!("[Err] Scanner Name Error\n\
                Please check is `scanners` in conf/Scanner.toml correct.\n\n\
                {}", _err);
                continue;
            }
        };
        
        // get scanner path
        let scanner_path = "scanner/".to_string() + &scanner_name;

        // get config
        let config_path = scanner_path.to_string() + "/Settings";
        let mut scanner_config = config::Config::default();
        match settings.merge(config::File::with_name(config_path.as_str())) {
            Ok(_) => {},
            Err(_err) => {
                println!("[Err] Scanner Config Error\n\
                    Please check {}.toml exists.\n\n\
                    {}", config_path, _err);
                continue;
            }
        };

        // get excutable file path
        let executable_path = match settings.get_str("excutable_file") {
            Ok(_excutable) => current_path.to_string() + "/" + &scanner_path.as_str().to_string() + "/" + &_excutable.to_string(),
            Err(_err) => {
                println!("[Err] Setting File Error\n\
                    Please check {}.toml has `excutable_file` string.\n\n\
                    {}", config_path, _err);
                continue;
            }
        };

        // make process and pipe
        let mut process = Command::new(executable_path);
        let mut pipe = process
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn().unwrap();
        
        // make scanner
        scanners.push(
            Scanner {
                name: scanner_name,
                path: scanner_path,
                config: scanner_config,
                process: process,
                pipe: pipe
            }
        );

        // for debug
        let mut scanner = scanners.pop().unwrap();
        let test: &str = "hello world";
        let bytes: &[u8] = test.as_bytes();
        let response = scanner.request(bytes);

        match response {
            Ok(_response) => {
                println!("{}", _response);
            },
            Err(_err) => {
                println!("{}", _err)
            }
        }
    }

    /* get signature list */
    // set arguments
    let signature_path = PathBuf::from("./signature");
    let mut signatures: Vec<PathBuf> = Vec::new();
    
    search_files(&mut signatures, signature_path);

    // print signatures
    /*
    println!("[Signature Path]");
    for signature in signatures {
        println!("Path: {}", signature.display());
    }
    */
}