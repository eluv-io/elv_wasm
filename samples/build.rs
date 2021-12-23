use std::process::Command;

pub fn execute(exe: &str, args: &[&str]) {
    Command::new(exe)
        .args(args)
        .spawn()
        .expect(&format!("failed to start external executable {}", exe));
}

fn main() {
    println!("handling building assemblyscript");
    execute("npm", &["install", "asc"]);
    execute("npm", &["run", "asbuild"]);
}
