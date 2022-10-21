use std::process::Command;
use std::env;

pub fn execute(exe: &str, args: &[&str]) {
    Command::new(exe)
        .args(args)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to start external executable {}", exe));
}

fn main() {
    println!("handling building assemblyscript");
    let do_asc = match env::var_os("BUILD_ASC"){
        Some(_) => true,
        None => false,
    };
    if do_asc{
        execute("npm", &["install", "asc"]);
        execute("npm", &["run", "asbuild"]);
    }
}
