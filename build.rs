#[allow(dead_code)]
// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const QT_VERSION: &'static str = "6.10.0";
const BASE_DIR: &'static str = ".";
const LIBDIE_BASE_DIR: &'static str = "./libdie++";
const LIBDIE_BUILD_DIR: &'static str = "./libdie++/build";
const LIBDIE_INSTALL_DIR: &'static str = "./libdie++/install";
const LIB_DIE_PATH: &'static str = "./libdie++/build/_deps/dielibrary-build/src";

#[cfg(target_os = "windows")]
const WINDOWS_KITS_LIB_DIR: &'static str = r"C:\Program Files (x86)\Windows Kits\10\Lib";

#[cfg(debug_assertions)]
const BUILD_TYPE: &'static str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &'static str = "Release";

fn get_qt_libs_path() -> String {
    #[cfg(target_os = "windows")]
    return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/msvc2022_64/lib");

    #[cfg(target_os = "macos")]
    return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/macos/lib");

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "aarch64")]
        return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/gcc_arm64/lib");

        #[cfg(target_arch = "x86_64")]
        return format!("./{LIBDIE_BUILD_DIR}/{QT_VERSION}/gcc_64/lib");
    }
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
        {
            #[cfg(target_arch = "x86_64")]
            cmd.args(["linux", "desktop", QT_VERSION]);

            #[cfg(target_arch = "aarch64")]
            cmd.args(["linux_arm64", "desktop", QT_VERSION, "linux_gcc_arm64"]);
        }
        #[cfg(target_os = "macos")]
        cmd.args(["mac", "desktop", QT_VERSION]);
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

fn has_windows_ucrt_libs(path: &Path) -> bool {
    path.join("ucrt").join("x64").exists()
}

fn normalized_windows_sdk_version(version: &str) -> &str {
    version.trim_end_matches(['\\', '/'])
}

fn parse_windows_sdk_version(version: &str) -> Option<Vec<u32>> {
    normalized_windows_sdk_version(version)
        .split('.')
        .map(|segment| segment.parse::<u32>().ok())
        .collect()
}

fn find_latest_windows_sdk_dir(base_dir: &Path) -> Option<PathBuf> {
    fs::read_dir(base_dir)
        .ok()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_dir() || !has_windows_ucrt_libs(&path) {
                return None;
            }

            let version = entry.file_name();
            let version = version.to_str()?;
            let version = parse_windows_sdk_version(version)?;

            Some((version, path))
        })
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, path)| path)
}

#[cfg(target_os = "windows")]
fn resolve_windows_sdk_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("MSVC_PATH") {
        let path = PathBuf::from(path);
        if has_windows_ucrt_libs(&path) {
            return Some(path);
        }
    }

    if let Some(root) = env::var_os("WindowsSdkDir") {
        let version = env::var("WindowsSDKLibVersion")
            .or_else(|_| env::var("WindowsSDKVersion"))
            .ok();

        if let Some(version) = version {
            let path = PathBuf::from(root)
                .join("Lib")
                .join(normalized_windows_sdk_version(&version));

            if has_windows_ucrt_libs(&path) {
                return Some(path);
            }
        }
    }

    find_latest_windows_sdk_dir(Path::new(WINDOWS_KITS_LIB_DIR))
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
    println!("cargo:rerun-if-env-changed=MSVC_PATH");
    println!("cargo:rerun-if-env-changed=WindowsSdkDir");
    println!("cargo:rerun-if-env-changed=WindowsSDKLibVersion");
    println!("cargo:rerun-if-env-changed=WindowsSDKVersion");

    match BUILD_TYPE {
        "Release" => {
            println!("cargo:rustc-link-lib=static=Qt6Core");
            println!("cargo:rustc-link-lib=static=Qt6Qml");
            println!("cargo:rustc-link-lib=static=Qt6Network");
            println!("cargo:rustc-link-lib=dylib=Qt6Core");
            println!("cargo:rustc-link-lib=dylib=Qt6Qml");
            println!("cargo:rustc-link-lib=dylib=Qt6Network");
        }
        "Debug" => {
            let windows_sdk_dir = resolve_windows_sdk_dir().expect(
                "failed to locate Windows SDK ucrt path; set MSVC_PATH or install a Windows 10 SDK",
            );
            let ucrt_dir = windows_sdk_dir.join("ucrt").join("x64");

            println!("cargo:rustc-link-lib=static=Qt6Cored");
            println!("cargo:rustc-link-lib=static=Qt6Qmld");
            println!("cargo:rustc-link-lib=static=Qt6Networkd");
            println!("cargo:rustc-link-lib=dylib=Qt6Cored");
            println!("cargo:rustc-link-lib=dylib=Qt6Qmld");
            println!("cargo:rustc-link-lib=dylib=Qt6Networkd");
            println!("cargo:rustc-link-search=native={}", ucrt_dir.display());
            println!("cargo:rustc-link-lib=static=ucrtd");
        }
        _ => {
            unimplemented!()
        }
    };

    println!("cargo:rustc-link-search=native={}/die", LIBDIE_INSTALL_DIR);

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
}

fn is_qt_missing() -> bool {
    std::path::Path::new(get_qt_libs_path().as_str()).exists() == false
}

fn should_rebuild_libdie() -> bool {
    for _mod in ["bzip2", "lzma", "zlib"].iter() {
        #[cfg(target_os = "windows")]
        let path_str = format!("{}/XArchive/3rdparty/{}/{}", LIB_DIE_PATH, _mod, BUILD_TYPE);

        #[cfg(not(target_os = "windows"))]
        let path_str = format!("{}/XArchive/3rdparty/{}", LIB_DIE_PATH, _mod);

        if !std::path::Path::new(path_str.as_str()).exists() {
            return true;
        }
    }

    let mut fpath = std::path::PathBuf::from(LIBDIE_INSTALL_DIR);

    #[cfg(target_os = "windows")]
    fpath.push("die.lib");

    #[cfg(target_os = "linux")]
    fpath.push("lib/libdie.a");

    #[cfg(target_os = "macos")]
    fpath.push("lib/libdie.a");

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("die-rust-build-rs-{name}-{timestamp}"))
    }

    #[test]
    fn normalizes_windows_sdk_version() {
        assert_eq!(
            normalized_windows_sdk_version("10.0.26100.0\\"),
            "10.0.26100.0"
        );
        assert_eq!(
            normalized_windows_sdk_version("10.0.26100.0/"),
            "10.0.26100.0"
        );
    }

    #[test]
    fn finds_latest_windows_sdk_dir() {
        let base_dir = temp_test_dir("windows-sdk");
        let older = base_dir.join("10.0.22000.0").join("ucrt").join("x64");
        let newer = base_dir.join("10.0.26100.0").join("ucrt").join("x64");
        let invalid = base_dir.join("invalid").join("ucrt").join("x64");

        fs::create_dir_all(&older).unwrap();
        fs::create_dir_all(&newer).unwrap();
        fs::create_dir_all(&invalid).unwrap();

        let latest = find_latest_windows_sdk_dir(&base_dir).unwrap();
        assert_eq!(latest, base_dir.join("10.0.26100.0"));

        fs::remove_dir_all(&base_dir).unwrap();
    }
}
