use std::time;
use std::thread;
use std::process::Command;

fn update_repository() {
    // pull submodules 
    Command::new("sh")
        .arg("-c")
        .arg("git submodule foreach git pull origin master")
        .current_dir("./signature")
        .spawn()
        .expect("[note] pull submodule failed\n");
}

pub fn auto_pull() {
    // repeat pull
    thread::spawn(move || {
        loop {
            update_repository();
            
            let ten_minits = time::Duration::from_secs(60);
            thread::sleep(ten_minits);
        }
    });
}