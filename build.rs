#[allow(dead_code)]
// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html
use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::fs;

const QT_VERSION: &str = "6.10.0";
const BASE_DIR: &str = ".";
const LIBDIE_BASE_DIR: &str = "libdie++";
const LIBDIE_BUILD_DIR: &str = "libdie++/build";
const LIBDIE_INSTALL_DIR: &str = "libdie++/install";
const LIB_DIE_PATH: &str = "libdie++/build/_deps/dielibrary-build/src";

#[cfg(target_os = "windows")]
const WINDOWS_KITS_LIB_DIR: &str = r"C:\Program Files (x86)\Windows Kits\10\Lib";

#[cfg(debug_assertions)]
const BUILD_TYPE: &str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &str = "Release";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"))
}

/// Resolve `path` against the crate root, leaving it alone if it is already absolute
fn absolute(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();

    if path.is_absolute() {
        path.display().to_string()
    } else {
        manifest_dir().join(path).display().to_string()
    }
}

/// Absolute path of the Qt6 libraries downloaded and managed by [`qt_download`]
fn managed_qt_libs_path() -> String {
    #[cfg(target_os = "windows")]
    return absolute(format!("{LIBDIE_BUILD_DIR}/{QT_VERSION}/msvc2022_64/lib"));

    #[cfg(target_os = "macos")]
    return absolute(format!("{LIBDIE_BUILD_DIR}/{QT_VERSION}/macos/lib"));

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "aarch64")]
        return absolute(format!("{LIBDIE_BUILD_DIR}/{QT_VERSION}/gcc_arm64/lib"));

        #[cfg(target_arch = "x86_64")]
        return absolute(format!("{LIBDIE_BUILD_DIR}/{QT_VERSION}/gcc_64/lib"));
    }
}

fn get_qt_libs_path() -> String {
    match env::var("QT6_LIB_PATH") {
        Ok(path) if !path.trim().is_empty() => absolute(path.trim()),
        _ => managed_qt_libs_path(),
    }
}

fn qt_cmake_config_dir() -> String {
    format!("{}/cmake/Qt6", get_qt_libs_path().replace('\\', "/"))
}

fn qt_download() {
    // Install AQT
    {
        assert!(
            std::process::Command::new("python")
                .current_dir(BASE_DIR)
                .args(["-m", "pip", "install", "--upgrade", "aqtinstall"])
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
                .unwrap_or_else(|_| panic!("failed to install Qt {QT_VERSION} using AQT"))
                .success()
        );
    }
}

fn cmake_build_die() {
    // CMake configure
    {
        let qt_cmake_dir = qt_cmake_config_dir();

        assert!(
            Path::new(&qt_cmake_dir).is_dir(),
            "'{qt_cmake_dir}' does not exist: QT6_LIB_PATH must point to a Qt6 library directory \
             containing cmake/Qt6/Qt6Config.cmake"
        );

        assert!(
            std::process::Command::new("cmake")
                .args(["-S", LIBDIE_BASE_DIR])
                .args(["-B", LIBDIE_BUILD_DIR])
                .arg(format!("-DQt6_DIR:PATH={qt_cmake_dir}"))
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

#[cfg(target_os = "windows")]
fn has_windows_ucrt_libs(path: &Path) -> bool {
    path.join("ucrt").join("x64").exists()
}

#[cfg(target_os = "windows")]
fn normalized_windows_sdk_version(version: &str) -> &str {
    version.trim_end_matches(['\\', '/'])
}

#[cfg(target_os = "windows")]
fn parse_windows_sdk_version(version: &str) -> Option<Vec<u32>> {
    normalized_windows_sdk_version(version)
        .split('.')
        .map(|segment| segment.parse::<u32>().ok())
        .collect()
}

#[cfg(target_os = "windows")]
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
    println!("cargo:rustc-link-search=native={}", get_qt_libs_path());

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
    let install_dir = absolute(LIBDIE_INSTALL_DIR);
    let lib_die_path = absolute(LIB_DIE_PATH);
    let qt_lib_path = get_qt_libs_path();

    println!("cargo:rustc-link-search=native={install_dir}/die");
    println!("cargo:rustc-link-search=native={install_dir}/die/lib");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=Qt6Core");
    println!("cargo:rustc-link-lib=dylib=Qt6Qml");
    println!("cargo:rustc-link-lib=dylib=Qt6Network");

    println!("cargo:rustc-link-search=native={lib_die_path}/XCapstone");
    for _mod in ["bzip2", "lzma", "zlib"] {
        println!("cargo:rustc-link-search=native={lib_die_path}/XArchive/3rdparty/{_mod}");
    }

    println!("cargo:rustc-link-arg=-Wl,-rpath,{install_dir}/die/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{qt_lib_path}");
}

#[cfg(target_os = "macos")]
fn install() {
    let install_dir = absolute(LIBDIE_INSTALL_DIR);
    let lib_die_path = absolute(LIB_DIE_PATH);
    let qt_lib_path = get_qt_libs_path();

    println!("cargo:rustc-link-search=native={install_dir}/die");
    println!("cargo:rustc-link-search=native={install_dir}/die/lib");
    println!("cargo:rustc-link-lib=dylib=c++");

    println!("cargo:rustc-link-search=framework={qt_lib_path}/");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{qt_lib_path}");

    println!("cargo:rustc-link-lib=framework=QtCore");
    println!("cargo:rustc-link-lib=framework=QtQml");
    println!("cargo:rustc-link-lib=framework=QtNetwork");

    println!("cargo:rustc-link-search=native={lib_die_path}/XCapstone");
    for _mod in ["bzip2", "lzma", "zlib"] {
        println!("cargo:rustc-link-search=native={lib_die_path}/XArchive/3rdparty/{_mod}");
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

    let install_dir = absolute(LIBDIE_INSTALL_DIR);
    let build_dir = absolute(LIBDIE_BUILD_DIR);
    let lib_die_path = absolute(LIB_DIE_PATH);

    println!("cargo:rustc-link-search=native={install_dir}/die");
    println!("cargo:rustc-link-search=native={install_dir}/die/dielib");

    println!("cargo:rustc-link-search=native={build_dir}/{BUILD_TYPE}");
    println!(
        "cargo:rustc-link-search=native={build_dir}/_deps/dielibrary-build/src/dielib/{BUILD_TYPE}"
    );
    for _mod in ["bzip2", "lzma", "zlib"] {
        println!(
            "cargo:rustc-link-search=native={lib_die_path}/XArchive/3rdparty/{_mod}/{BUILD_TYPE}"
        );
    }
    println!("cargo:rustc-link-search=native={lib_die_path}/XCapstone/{BUILD_TYPE}");
    println!("cargo:rustc-link-lib=dylib=Crypt32");
    println!("cargo:rustc-link-lib=dylib=Wintrust");
}

fn is_qt_missing() -> bool {
    !Path::new(get_qt_libs_path().as_str()).exists()
}

fn should_rebuild_libdie() -> bool {
    for _mod in ["bzip2", "lzma", "zlib"] {
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

    !fpath.exists()
}

fn main() {
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    unimplemented!();

    if is_qt_missing() {
        qt_download();
    }

    let qt_lib_path = get_qt_libs_path();
    assert!(
        Path::new(&qt_lib_path).is_dir(),
        "'{qt_lib_path}' is not an existing directory, check the QT6_LIB_PATH environment variable"
    );

    if should_rebuild_libdie() {
        cmake_build_die();
    }

    setup_common();
    install();

    println!("cargo:qt_lib_path={qt_lib_path}");

    #[cfg(target_os = "windows")]
    let install_lib_path = format!("{}/die", absolute(LIBDIE_INSTALL_DIR));

    #[cfg(not(target_os = "windows"))]
    let install_lib_path = format!("{}/die/lib", absolute(LIBDIE_INSTALL_DIR));

    println!("cargo:install_lib_path={install_lib_path}");

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
