use std::{env, fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=build.rs");
    let mut files = vec!["Cargo.lock".into(), "build.rs".into()];
    collect(Path::new("src"), &mut files);
    files.sort();
    let mut hash = 14695981039346656037_u64;
    for path in files {
        for byte in fs::read(path).expect("read build input") {
            hash = (hash ^ byte as u64).wrapping_mul(1099511628211);
        }
    }
    let rustc = Command::new(env::var_os("RUSTC").expect("RUSTC"))
        .arg("--version")
        .output()
        .expect("rustc version");
    let compiler = String::from_utf8(rustc.stdout).expect("UTF-8 rustc version");
    let id = format!(
        "{}|{}|{}|{:016x}",
        env::var("TARGET").unwrap(),
        env::var("PROFILE").unwrap(),
        compiler.trim(),
        hash
    );
    println!("cargo:rustc-env=IGNIS_BUILD_ID={id}");
}
fn collect(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory") {
        let p = entry.expect("source entry").path();
        if p.is_dir() {
            collect(&p, files);
        } else {
            files.push(p);
        }
    }
}
