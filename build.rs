use std::env;
use std::process::Command;

pub fn execute(exe: &str, args: &[&str]) {
    Command::new(exe)
        .args(args)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to start external executable {exe}"));
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("handling building assemblyscript");
    let mut pbase = env::current_dir().unwrap();
    pbase.push("samples");
    env::set_current_dir(pbase.clone()).unwrap();
    let do_asc = env::var_os("BUILD_ASC").is_some();
    if do_asc {
        execute("npm", &["install", "asc"]);
        execute("npm", &["run", "asbuild"]);
    }
    pbase.push("go");
    pbase.push("test_wapc");

    env::set_current_dir(pbase).unwrap();
    execute(
        "tinygo",
        &[
            "build",
            "-o",
            "test_wapc.wasm",
            "-target=wasi",
            "-no-debug",
            "main.go",
        ],
    );
    execute(
        "mv",
        &[
            "-f",
            "-u",
            "test_wapc.wasm",
            "../../../target/wasm32-unknown-unknown/release/",
        ],
    )
}
