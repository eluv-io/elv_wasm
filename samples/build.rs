use std::env;
use std::process::Command;

pub fn execute(exe: &str, args: &[&str]) {
    let status = Command::new(exe)
        .args(args)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to start external executable {exe}"))
        .wait()
        .expect("failed to wait on child process");
    if !status.success() {
        panic!("external executable {exe} failed with status: {}", status);
    }
}

fn main() {
    println!("handling building assemblyscript");
    let mut pbase = env::current_dir().unwrap();
    let do_asc = env::var_os("BUILD_ASC").is_some();
    if do_asc {
        pbase.push("samples");
        env::set_current_dir(&pbase).unwrap();
        execute("npm", &["install", "asc"]);
        execute("npm", &["run", "asbuild"]);
        pbase.pop();
    }

    pbase.push("go");
    pbase.push("test_wapc");
    println!("path={pbase:?}");

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
