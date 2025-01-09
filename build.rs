// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html

const BASE_DIR: &'static str = "./libdie++";
#[allow(dead_code)]
const BUILD_DIR: &'static str = "./libdie++/build";
const INSTALL_DIR: &'static str = "./libdie++/install";
const LIB_DIE_PATH: &'static str = "./libdie++/build/_deps/dielibrary-build/src";

#[cfg(target_os = "windows")]
const MSVC_PATH: &'static str = r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22000.0";

#[cfg(debug_assertions)]
const BUILD_TYPE: &'static str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &'static str = "Release";

fn cmake_build_die() {
    // CMake configure
    {
        assert!(std::process::Command::new("cmake")
            .current_dir(BASE_DIR)
            .args(["-S", "."])
            .args(["-B", "build"])
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to configure cmake")
            .success());
    }

    // CMake build
    {
        let nb_cpu = "4";

        assert!(std::process::Command::new("cmake")
            .args(["--build", "build"])
            .args(["-j", nb_cpu])
            .args(["--config", BUILD_TYPE])
            .current_dir(BASE_DIR)
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to build with cmake")
            .success());
    }

    // CMake install
    {
        assert!(std::process::Command::new("cmake")
            .args(["--install", "build"])
            .args(["--config", BUILD_TYPE])
            .args(["--prefix", "install"])
            .current_dir(BASE_DIR)
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to install with cmake")
            .success());
    }
}

fn install_common() {
    // die & die++
    println!("cargo:rustc-link-lib=static=die++");
    println!("cargo:rustc-link-lib=static=die");

    // die 3rd party
    println!("cargo:rustc-link-lib=static=bzip2");
    println!("cargo:rustc-link-lib=static=lzma");
    println!("cargo:rustc-link-lib=static=zlib");
    println!("cargo:rustc-link-lib=static=capstone_x86");

    // qt
    println!("cargo:rerun-if-env-changed=QT6_LIB_PATH");
    if let Some(qt_lib_path) = option_env!("QT6_LIB_PATH") {
        println!("cargo:rustc-link-search=native={}", qt_lib_path);
    }

    if BUILD_TYPE == "Release" {
        println!("cargo:rustc-link-lib=static=Qt6Core");
        println!("cargo:rustc-link-lib=static=Qt6Qml");
        println!("cargo:rustc-link-lib=static=Qt6Network");

        println!("cargo:rustc-link-lib=dylib=Qt6Core");
        println!("cargo:rustc-link-lib=dylib=Qt6Qml");
        println!("cargo:rustc-link-lib=dylib=Qt6Network");
    }
}

#[cfg(target_os = "linux")]
fn install_linux() {
    println!("cargo:rustc-link-search=native={}/die", INSTALL_DIR);
    println!("cargo:rustc-link-search=native={}/die/lib", INSTALL_DIR);
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=Qt6Core");
    println!("cargo:rustc-link-lib=dylib=Qt6Qml");
    println!("cargo:rustc-link-lib=dylib=Qt6Network");
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    println!("cargo:rustc-link-search=native={}/XCapstone", LIB_DIE_PATH);
    for _mod in ["bzip2", "lzma", "zlib"].iter() {
        println!(
            "cargo:rustc-link-search=native={}/XArchive/3rdparty/{}",
            LIB_DIE_PATH, _mod
        );
    }
}

#[cfg(target_os = "macos")]
fn install_macos() {
    todo!();
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
}

#[cfg(target_os = "windows")]
fn install_windows() {
    println!("cargo:rustc-link-search=native={}/die/dielib", INSTALL_DIR);
    println!(
        "cargo:rustc-link-search=native={}/{}",
        BUILD_DIR, BUILD_TYPE
    );
    println!(
        "cargo:rustc-link-search=native={}/_deps/dielibrary-build/src/dielib/{}",
        BUILD_DIR, BUILD_TYPE
    );
    for _mod in ["bzip2", "lzma", "zlib"].iter() {
        println!(
            "cargo:rustc-link-search=native={}/XArchive/3rdparty/{}/{}",
            LIB_DIE_PATH, _mod, BUILD_TYPE
        );
    }
    println!(
        "cargo:rustc-link-search=native={}/XCapstone/{}",
        LIB_DIE_PATH, BUILD_TYPE
    );
    println!("cargo:rustc-link-lib=dylib=Crypt32");
    println!("cargo:rustc-link-lib=dylib=Wintrust");

    if BUILD_TYPE == "Debug" {
        println!("cargo:rustc-link-lib=static=Qt6Cored");
        println!("cargo:rustc-link-lib=static=Qt6Qmld");
        println!("cargo:rustc-link-lib=static=Qt6Networkd");
        println!("cargo:rustc-link-lib=dylib=Qt6Cored");
        println!("cargo:rustc-link-lib=dylib=Qt6Qmld");
        println!("cargo:rustc-link-lib=dylib=Qt6Networkd");

        println!("cargo:rustc-link-search=native={}/ucrt/x64", MSVC_PATH);
        println!("cargo:rustc-link-lib=static=ucrtd");
    }
}

fn main() {
    cmake_build_die();

    install_common();

    if cfg!(target_os = "linux") {
        install_linux();
    } else if cfg!(target_os = "macos") {
        #[cfg(target_os = "macos")]
        install_macos();
    } else if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        install_windows();
    } else {
        unimplemented!();
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
}
