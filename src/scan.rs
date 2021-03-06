extern crate base64;
extern crate config;
extern crate serde_json;

use std::vec::Vec;
use std::fs;
use std::io::Read;
use std::env;
use std::path::PathBuf;

use scanner::Scanner;
use scan_utils::*;


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

        if path.file_name().unwrap().to_str().unwrap().to_string().starts_with(".") {
            // do not push hidden files
            continue;
        }

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


/// Search scanner list,
/// and return them built.
fn build_scanners(target: ScanTarget) -> Result<Vec<Scanner>, String> {
    /* Get scanner */
    let mut scanners: Vec<Scanner> = Vec::new();

    // get current directory
    let unwraped_current_path = env::current_dir().unwrap();
    let current_path = unwraped_current_path.to_str().unwrap();

    // read config
    let mut settings = config::Config::new();
    match settings.merge(config::File::with_name("conf/Scanner")) {
        Ok(_) => {},
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Scanner.toml exists.\n\n\
                {}", _err);
            return Err("unexpected err".to_string());
        }
    }

    // get scanner names
    let scanners_names = match settings.get_array("scanners") {
        Ok(_scanners) => _scanners,
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Server.toml has `scanners` array.\n\n\
                {}", _err);
            return Err("unexpected err".to_string());
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
        let mut scanner_config = config::Config::new();
        match scanner_config.merge(config::File::with_name(config_path.as_str())) {
            Ok(_) => {},
            Err(_err) => {
                println!("[Err] Scanner Config Error\n\
                    Please check {}.toml exists.\n\n\
                    {}", config_path, _err);
                continue;
            }
        };
        // get excutable file path
        let executable_path = match scanner_config.get_str("excutable_file") {
            Ok(_excutable) => current_path.to_string() + "/" + &scanner_path.as_str().to_string() + "/" + &_excutable.to_string(),
            Err(_err) => {
                println!("[Err] Setting File Error\n\
                    Please check {}.toml has `excutable_file` string.\n\n\
                    {}", config_path, _err);
                continue;
            }
        };

        // make scanner
        let mut scanner = Scanner::new(
            scanner_name.clone(),
            scanner_config,      
            scanner_path,
            executable_path
        );

        match scanner.request(target.target.clone()) {
            Ok(_) => {},
            Err(_err) => {
                println!("[Unexpected Err] Scanner Err\n\
                    Please check scanner {} works correct.\n\n\
                    {}", scanner_name, _err);
                continue;
            }
        };

        scanners.push(scanner);
    }

    return Ok(scanners);
}

/// Using signatures and scanner to scan target.
pub fn scan(target: ScanTarget) -> Result<ScanResult, String> {
    let mut scan_result = ScanResult::new();

    // get scanner list
    let mut scanners = match build_scanners(target) {
        Ok(_scanner) => _scanner,
        Err(_err) => {
            println!("{}", _err);
            return Err(_err);
        }
    };

    // get signature list
    let signature_path = PathBuf::from("./signature");
    let mut signatures: Vec<PathBuf> = Vec::new();
    
    search_files(&mut signatures, signature_path);

    for path in &mut signatures {
        // get extention
        let extention = path.extension().unwrap().to_str().unwrap();

        /* Read signature files */
        // open signature file
        let mut f = match fs::File::open(path.clone()) {
            Ok(_f) => _f,
            Err(_err) => {
                println!("[Unexpected Err] Open Signature Err\n\
                    Please check the {} openable.\n\n\
                    {}", path.clone().display(), _err);
                continue;
            }
        };

        // read signature file
        let mut signature = Vec::new();
        match f.read_to_end(&mut signature){
            Ok(_) => {},
            Err(_err) => {
                println!("[Unexpected Err] Read Signature Err\n\
                    Please check the {} readable.\n\n\
                    {}", path.display(), _err);
                continue;
            }
        }

        
        // set signatures
        for scanner in &mut scanners {
            // get extensions of config
            let extensions = match scanner.config.get_array("extensions") {
                Ok(_extensions) => _extensions,
                Err(_err) => {
                    println!("[Unexpected Err] Read Extension Err\n\
                        Please check the {} extension exits.\n\n\
                        {}", scanner.name, _err);
                    scanner.kill();
                    continue;
                }
            };

            /* Send signatures */
            // check signature extention in
            // extentions of scanner config
            for _extention in extensions {
                if extention == _extention.into_str().unwrap() {
                    // if extentions same,
                    // send signature
                    match scanner.request_by_bytes(&mut signature) {
                        Ok(_) => {
                        },
                        Err(_err) => {
                            print!("{}", _err);
                            scanner.kill();
                        }
                    }

                    break;
                }
            }
        }
    }

    /* Get Result From Scanner */
    for scanner in &mut scanners {
        // get result
        let result = match scanner.request_end() {
            Ok(_result) => _result,
            Err(_err) => {
                println!("{}", _err);
                scanner.kill();
                continue;
            }
        };

        // kill process
        scanner.kill();

        // parse json
        let result: serde_json::value::Value = match serde_json::from_str(&result) {
            Ok(_result) => _result,
            Err(_err) => {
                println!("[Unexpected Err] Read Result Err\n\
                    Please check the {}'s output correct.\n\n\
                    {}", scanner.name, _err);
                continue;
            }
        };

        // get hit
        let hit = match result["hit"].as_bool() {
            Some(_hit) => _hit,
            None => {
                println!("[Unexpected Err] Scanner Result Err\n\
                    Please check the {}'s output correct.\n\n",
                    scanner.name);
                continue;
            }
        };

        // get message
        let value_messages = match result["messages"].as_array() {
            Some(_messages) => _messages,
            None => {
                println!("[Unexpected Err] Scanner Result Err\n\
                    Please check the {}'s output correct.\n\n",
                    scanner.name);
                continue;
            }
        };

        // convert value to string
        let mut messages = Vec::new();

        for value in value_messages {
            let value = match value.as_str() {
                Some(_messages) => _messages,
                None => {
                    println!("[Unexpected Err] Scanner Result Err\n\
                        Please check the {}'s output correct.\n\n",
                        scanner.name);
                    continue;
                }
            };

            messages.push(value.to_string());
        }

        // return result
        let mut result = ScanResult::init(hit, true, messages);
        scan_result.add(&mut result);
    }

    return Ok(
        scan_result
    )
}