use std::vec::Vec;
use std::fs;
use std::path::PathBuf;

extern crate config;


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
    /* get scanner */
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

    let scanners_names = match settings.get_array("scanners") {
        Ok(_scanners) => _scanners,
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Server.toml has `scanners` array.\n\n\
                {}", _err);
            return;
        }
    };

    // print signatures
    println!("[Scanner Name]");
    for scanner_name in scanners_names {
        println!("Name: {}", scanner_name);
    }

    /* get signature list */
    // set arguments
    let signature_path = PathBuf::from("./signature");
    let mut signatures: Vec<PathBuf> = Vec::new();
    
    search_files(&mut signatures, signature_path);

    // print signatures
    println!("[Signature Path]");
    for signature in signatures {
        println!("Path: {}", signature.display());
    }
}