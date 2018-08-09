extern crate base64;
extern crate config;
extern crate serde_json;

use std::vec::Vec;
use std::fs;
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
pub fn scan(target: ScanTarget) -> Result<ScanResult, String> {
    /* Get scanner */
    let mut scanners: Vec<Scanner> = Vec::new();
    let mut scan_result = ScanResult::new();

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

        // make scanner
        scanners.push(
            Scanner::new(
                scanner_name,
                scanner_config,      
                scanner_path,
                executable_path
            )
        );

        /* for debug */
        // scanner.request()
        let mut scanner = scanners.pop().unwrap();
        let message = target.target.clone();
        
        let response = scanner.request(message);
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

    scan_result.message = String::from("hello world !");

    return Ok(
        scan_result
    )
}