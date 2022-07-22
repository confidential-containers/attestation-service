use std::process::{exit, Command};

fn real_main() -> Result<(), String> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed={}", out_dir);
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=intoto");
    let intoto_dir = "src/intoto".to_string();
    let intoto = Command::new("go")
        .args(&[
            "build",
            "-o",
            &format!("{}/libintoto.a", out_dir),
            "-buildmode=c-archive",
            "intoto.go",
        ])
        .current_dir(intoto_dir)
        .output()
        .expect("failed to launch intoto compile process");
    if !intoto.status.success() {
        return Err(std::str::from_utf8(&intoto.stderr.to_vec())
            .unwrap()
            .to_string());
    }

    Ok(())
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("ERROR: {}", e);
        exit(1);
    }
}
