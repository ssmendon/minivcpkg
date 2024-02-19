//! A small build dependency for locating vcpkg packages.

use std::{
    ffi::OsString,
    fmt::Display,
    path::{Path, PathBuf},
};

struct Config {
    name: String,
    target: Option<String>,
    host: Option<String>,
    out_dir: Option<PathBuf>,
    static_crt: Option<bool>,
    vcpkg_target: Option<OsString>,
    vcpkg_host: Option<OsString>,
    vcpkg_root: Option<PathBuf>,
    vcpkg_tree_root: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Error {
    message: String,
    kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

#[derive(Debug)]
enum ErrorKind {
    Placeholder,
}

impl std::error::Error for Error {}

fn env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

impl Config {
    pub fn new(name: &str) -> Config {
        Config {
            name: name.to_owned(),
            target: None,
            host: None,
            out_dir: None,
            static_crt: None,
            vcpkg_root: None,
            vcpkg_tree_root: None,
            vcpkg_target: None,
            vcpkg_host: None,
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_owned());

        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_owned());

        self
    }

    pub fn out_dir(mut self, path: &Path) -> Self {
        self.out_dir = Some(path.to_owned());

        self
    }

    pub fn static_crt(mut self, is_static: bool) -> Self {
        self.static_crt = Some(is_static);

        self
    }

    pub fn vcpkg_root(mut self, path: &Path) -> Self {
        self.vcpkg_root = Some(path.to_owned());

        self
    }

    pub fn vcpkg_host(mut self, triplet: &str) -> Self {
        self.vcpkg_host = Some(triplet.into());

        self
    }

    pub fn vcpkg_target(mut self, triplet: &str) -> Self {
        self.vcpkg_target = Some(triplet.into());

        self
    }

    pub fn locate(mut self) -> Result<(), crate::Error> {
        // Using map_or_else to add error information later.
        let vcpkg_installed_root = self
            .vcpkg_tree_root
            .map_or_else(|| self.vcpkg_root.map(|r| r.join("installed")), Some)
            .map_or_else(|| env("VCPKG_ROOT").map(PathBuf::from), Some)
            .map_or_else(
                || {
                    std::env::current_dir()
                        .ok()
                        .map(|x| x.join("vcpkg_installed"))
                },
                Some,
            )
            .map_or_else(|| env("CARGO_MANIFEST_DIR").map(PathBuf::from), Some)
            .filter(|x| x.exists() && x.is_dir());

        let vcpkg_installed_root = vcpkg_installed_root.unwrap();

        println!("{}", vcpkg_installed_root.display());

        // Determine the vcpkg target for this build.
        // TODO: Use the triplet environment variables and the default, or convert from cargo.
        // https://learn.microsoft.com/en-us/vcpkg/users/config-environment
        // https://doc.rust-lang.org/nightly/rustc/platform-support.html
        // https://llvm.org/docs/LangRef.html#target-triple
        // https://doc.rust-lang.org/reference/conditional-compilation.html
        // https://learn.microsoft.com/en-us/vcpkg/users/triplets
        // https://learn.microsoft.com/en-us/vcpkg/concepts/triplets

        let vcpkg_target = self
            .vcpkg_target
            .map_or_else(|| env("VCPKG_TARGET_TRIPLET"), Some)
            .map_or_else(|| env("VCPKG_DEFAULT_TRIPLET"), Some);

        let vcpkg_target = vcpkg_target.unwrap();
        let vcpkg_target = PathBuf::from(vcpkg_target);

        let base = vcpkg_installed_root.join(vcpkg_target);

        // https://learn.microsoft.com/en-us/vcpkg/users/buildsystems/manual-integration
        // https://github.com/mcgoo/vcpkg-rs/blob/master/src/lib.rs#L423-L426
        // TODO: fix this up with all the paths

        let lib = base.join("lib");
        let bin = base.join("bin");
        let include = base.join("include");

        // Emit metadata
        // todo: handle edge cases, best practices for -L, etc.
        // also no debugging libraries in this.
        // https://github.com/mcgoo/vcpkg-rs/blob/master/src/lib.rs#L1206-L1229
        // https://doc.rust-lang.org/cargo/reference/build-scripts.html
        println!("cargo:rustc-link-lib={}", self.name);
        println!("cargo:rustc-link-search=native={}", lib.display());
        println!("cargo:include={}", include.display());

        Ok(())
    }
}
