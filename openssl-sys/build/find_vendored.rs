use libressl_src as src;

use std::path::PathBuf;

pub fn get_openssl(_target: &str) -> (PathBuf, PathBuf) {
    let artifacts = src::Build::new().build();
    println!("cargo:vendored=1");
    println!(
        "cargo:root={}",
        artifacts.lib_dir().parent().unwrap().display()
    );

    (
        artifacts.lib_dir().to_path_buf(),
        artifacts.include_dir().to_path_buf(),
    )
}
