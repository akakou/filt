use std::vec::Vec;
use std::fs;
use std::path::PathBuf;


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
    // TODO

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