use std::process::Command;
use std::env;

pub fn execute(exe: &str, args: &[&str]) {
    Command::new(exe)
        .args(args)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to start external executable {exe}"));
}

fn main() {
    println!("handling building assemblyscript");
    let do_asc = env::var_os("BUILD_ASC").is_some();
    if do_asc{
        execute("npm", &["install", "asc"]);
        execute("npm", &["run", "asbuild"]);
    }
    let mut pbase = env::current_dir().unwrap();
    pbase.push("go");
    pbase.push("test_wapc");
    println!("path={:?}", pbase);

    env::set_current_dir(pbase).unwrap();
    execute("tinygo", &["build", "-o", "test_wapc.wasm", "-target=wasi", "-no-debug", "-scheduler=none",  "main.go"]);
    execute("cp", &["test_wapc.wasm", "-u", "../../../target/wasm32-unknown-unknown/release/"])
}
