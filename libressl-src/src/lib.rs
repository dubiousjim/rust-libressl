extern crate autotools;

use std::env;
use std::fs;
// use std::io::Write;
// use std::fs::File;
// use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("libressl")
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
}

pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    libs: Vec<String>,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("libressl-build")),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    /*
    fn cmd_make(&self) -> Command {
        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        if host.contains("dragonfly")
            || host.contains("freebsd")
            || host.contains("solaris")
            || host.contains("illumos")
        {
            Command::new("gmake")
        } else {
            Command::new("make")
        }
    }
    */

    pub fn build(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET dir not set")[..];
        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");
        let build_dir = out_dir.join("build");
        let install_dir = out_dir.clone(); // out_dir.join("install");

        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).unwrap();
        }

        let inner_dir = build_dir.join("src");
        fs::create_dir_all(&inner_dir).unwrap();
        cp_r(&source_dir(), &inner_dir);

        eprintln!("JIMP host={} {:?}", host, env::var_os("HOST"));
        eprintln!("JIMP target={} {:?}", target, env::var_os("TARGET"));
        eprintln!(
            "JIMP out_dir={} {:?}",
            out_dir.display(),
            env::var_os("OUT_DIR")
        );
        eprintln!("JIMP build_dir={}", build_dir.display());
        eprintln!("JIMP install_dir={}", install_dir.display());
        eprintln!("JIMP inner_dir={}", inner_dir.display());
        eprintln!("JIMP source_dir={}", source_dir().display());

        // see https://stackoverflow.com/a/33279062/272427
        let mut touch = Command::new("touch");
        touch.current_dir(&inner_dir);
        touch.arg("aclocal.m4");
        touch.arg("configure");
        touch.arg("Makefile.am");
        touch.arg("Makefile.in");
        self.run_command(touch, "touching ./configure etc for LibreSSL");

        use autotools::Config;
        let mut cfg = Config::new(&inner_dir);
        cfg.disable_shared();
        cfg.out_dir(&install_dir);
        if target.starts_with("i686-") && target.contains("-linux") {
            cfg.config_option("host", Some(target));
            /*
            cfg.free_host(target);
            // cfg.cflag("-m32");
            } else if target.ends_with("-musl") {
                cfg.free_host(target);
                // cfg.env("CC", "musl-gcc");
                // Ubuntu package linux-libc-dev-amd64-cross provides /usr/x86_64-linux-gnu/include/linux/sysctl.h
                // cfg.cflag("-I/usr/x86_64-linux-gnu/include");
            */
        }
        cfg.cflag("-v"); // JIMP
        let dst = cfg.build();
        eprintln!("JIMP result dst={}", dst.display());

        fs::remove_dir_all(&inner_dir).unwrap();

        let libs = if target.contains("msvc") {
            vec![
                "libtls".to_string(),
                "libssl".to_string(),
                "libcrypto".to_string(),
            ]
        } else {
            vec!["tls".to_string(), "ssl".to_string(), "crypto".to_string()]
        };

        Artifacts {
            lib_dir: install_dir.join("lib"),
            include_dir: install_dir.join("include"),
            libs: libs,
        }
    }

    #[allow(dead_code)]
    fn run_command(&self, mut command: Command, desc: &str) {
        println!("running {:?}", command);
        let status = command.status().unwrap();
        if !status.success() {
            panic!(
                "


Error {}:
    Command: {:?}
    Exit status: {}


    ",
                desc, command, status
            );
        }
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();

        // Skip git metadata as it's been known to cause issues and shouldn't be required
        if name.to_str() == Some(".git") {
            continue;
        }

        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

/*
fn apply_patches(target: &str, inner: &Path) {
    if !target.contains("musl") {
        return;
    }

    // Undo part of https://github.com/openssl/openssl/commit/c352bd07ed2ff872876534c950a6968d75ef121e on MUSL
    // since it doesn't have asm/unistd.h
    let mut buf = String::new();
    let path = inner.join("crypto/rand/rand_unix.c");
    File::open(&path).unwrap().read_to_string(&mut buf).unwrap();

    let buf = buf
        .replace("asm/unistd.h", "sys/syscall.h")
        .replace("__NR_getrandom", "SYS_getrandom");

    File::create(&path)
        .unwrap()
        .write_all(buf.as_bytes())
        .unwrap();
}
*/

/*
fn sanitize_sh(path: &Path) -> String {
    if !cfg!(windows) {
        return path.to_str().unwrap().to_string();
    }
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None;
        }
        if ch.next() != Some('/') {
            return None;
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}
*/

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}
