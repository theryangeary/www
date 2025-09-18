use std::{env, path::Path, process::Command};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);

    Command::new("cp")
        .arg("-R")
        .arg(manifest_path.join("static"))
        .arg(&out_path)
        .output()
        .expect("failed to copy /static to $OUT_DIR");

    // ./tailwindcss -i input.css -o static/output.css --minify
    Command::new("./tailwindcss")
        .arg("-i")
        .arg("input.css")
        .arg("-o")
        .arg(out_path.join("static").join("output.css"))
        .arg("--minify")
        .output()
        .expect("failed to compile tailwind styles");

    Command::new("rm")
        .arg(out_path.join("static").join("input.css"))
        .output()
        .expect("failed to remove input.css");

    let assets = Command::new("ls")
        .arg(out_path.join("static"))
        .output()
        .unwrap()
        .stdout;

    println!("cargo::warning=static-embedded assets: {:?}", String::from_utf8(assets));
}
