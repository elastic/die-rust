#[allow(dead_code)]
// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html
use std::env;

const QT_VERSION: &'static str = "6.10.0";
const BASE_DIR: &'static str = ".";
const LIBDIE_BASE_DIR: &'static str = "./libdie++";
const LIBDIE_BUILD_DIR: &'static str = "./libdie++/build";
const LIBDIE_INSTALL_DIR: &'static str = "./libdie++/install";
const LIB_DIE_PATH: &'static str = "./libdie++/build/_deps/dielibrary-build/src";

#[cfg(target_os = "windows")]
const MSVC_PATH: &'static str = r"C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22000.0";

#[cfg(debug_assertions)]
const BUILD_TYPE: &'static str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &'static str = "Release";

fn get_qt_libs_path() -> String {
    #[cfg(target_os = "windows")]
    return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/msvc2022_64/lib");

    #[cfg(target_os = "macos")]
    return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/clang_64/lib");

    #[cfg(target_os = "linux")]
    return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/gcc_64/lib");
}

fn qt_download() {
    // Install AQT
    {
        assert!(
            std::process::Command::new("python")
                .current_dir(BASE_DIR)
                .args(["-m", "pip", "install", "--user", "--upgrade", "aqtinstall"])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to install AQT")
                .success()
        );
    }

    // Install QT using AQT
    {
        let mut cmd = std::process::Command::new("python");
        cmd.current_dir(BASE_DIR)
            .args(["-m", "aqt", "install-qt", "-O", LIBDIE_BUILD_DIR]);

        #[cfg(target_os = "linux")]
        cmd.args(["linux", "desktop", QT_VERSION]);
        #[cfg(target_os = "macos")]
        cmd.args(["mac", "desktop", QT_VERSION, "clang_64"]);
        #[cfg(target_os = "windows")]
        cmd.args(["windows", "desktop", QT_VERSION, "win64_msvc2022_64"]);

        assert!(
            cmd.spawn()
                .unwrap()
                .wait()
                .expect(format!("failed to install Qt {QT_VERSION} using AQT").as_str())
                .success()
        );
    }

    // Add to env var
    {
        let fpath = get_qt_libs_path();

        println!("cargo:rustc-env=QT6_LIB_PATH=\"{fpath}\"");
        unsafe {
            env::set_var("QT6_LIB_PATH", fpath.as_str());
            env::set_var("Qt6_DIR", fpath.as_str());
        }
    }
}

fn cmake_build_die() {
    // CMake configure
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["-S", LIBDIE_BASE_DIR])
                .args(["-B", LIBDIE_BUILD_DIR])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to configure cmake")
                .success()
        );
    }

    // CMake build
    {
        let nb_cpu = "4";

        assert!(
            std::process::Command::new("cmake")
                .args(["--build", LIBDIE_BUILD_DIR])
                .args(["--parallel", nb_cpu])
                .args(["--config", BUILD_TYPE])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to build with cmake")
                .success()
        );
    }

    // CMake install
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["--install", LIBDIE_BUILD_DIR])
                .args(["--config", BUILD_TYPE])
                .args(["--prefix", LIBDIE_INSTALL_DIR])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to install with cmake")
                .success()
        );
    }
}

fn setup_common() {
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
fn install() {
    println!("cargo:rustc-link-search=native={}/die", LIBDIE_INSTALL_DIR);
    println!(
        "cargo:rustc-link-search=native={}/die/lib",
        LIBDIE_INSTALL_DIR
    );
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
fn install() {
    println!("cargo:rustc-link-search=native={}/die", LIBDIE_INSTALL_DIR);
    println!(
        "cargo:rustc-link-search=native={}/die/lib",
        LIBDIE_INSTALL_DIR
    );
    println!("cargo:rustc-link-lib=dylib=c++");

    if let Some(qt_lib_path) = option_env!("QT6_LIB_PATH") {
        println!("cargo:rustc-link-search=framework={}/", qt_lib_path);
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", qt_lib_path);
    }

    println!("cargo:rustc-link-lib=framework=QtCore");
    println!("cargo:rustc-link-lib=framework=QtQml");
    println!("cargo:rustc-link-lib=framework=QtNetwork");

    println!("cargo:rustc-link-search=native={}/XCapstone", LIB_DIE_PATH);
    for _mod in ["bzip2", "lzma", "zlib"].iter() {
        println!(
            "cargo:rustc-link-search=native={}/XArchive/3rdparty/{}",
            LIB_DIE_PATH, _mod
        );
    }
}

#[cfg(target_os = "windows")]
fn install() {
    println!(
        "cargo:rustc-link-search=native={}/die/dielib",
        LIBDIE_INSTALL_DIR
    );
    println!(
        "cargo:rustc-link-search=native={}/{}",
        LIBDIE_BUILD_DIR, BUILD_TYPE
    );
    println!(
        "cargo:rustc-link-search=native={}/_deps/dielibrary-build/src/dielib/{}",
        LIBDIE_BUILD_DIR, BUILD_TYPE
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

fn is_qt_missing() -> bool {
    std::path::Path::new(get_qt_libs_path().as_str()).exists() == false
}

fn should_rebuild_libdie() -> bool {
    let mut fpath = std::path::PathBuf::from(LIBDIE_INSTALL_DIR);

    #[cfg(target_os = "windows")]
    fpath.push("die.lib");

    #[cfg(target_os = "linux")]
    fpath.push("lib/libdie.a");

    #[cfg(target_os = "macos")]
    fpath.push("lib/");

    return fpath.exists() == false;
}

fn main() {
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    unimplemented!();

    if is_qt_missing() {
        qt_download();
    }

    if should_rebuild_libdie() {
        cmake_build_die();
    }

    setup_common();
    install();

    println!("cargo:rerun-if-changed=src/lib.rs");
}
