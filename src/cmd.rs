use std::process::Command;

pub fn init(_args: Vec<String>) {
    println!("Initializing new Forgery project from boilerplate...");

    let mut init = Command::new("forge")
        .arg("init")
        .arg("--template")
        .arg("Tudmotu/forgery-boilerplate")
        .spawn()
        .unwrap();

    init.wait().expect("forge init failed");
}
