// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    let base_dir = "./libdie++";
    let build_dir = "./libdie++/build";
    let install_dir = "./libdie++/install";
    let lib_die_path = "./libdie++/build/_deps/dielibrary-build/src";

    // CMake configure
    {
        std::process::Command::new("cmake")
            .args(["-S", base_dir])
            .args(["-B", build_dir])
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to configure cmake");
    }

    // CMake build
    {
        let nb_cpu = "4";

        std::process::Command::new("cmake")
            .args(["--build", build_dir])
            .args(["-j", nb_cpu])
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to build with cmake");
    }

    // CMake install
    {
        std::process::Command::new("cmake")
            .args(["--install", build_dir])
            .args(["--prefix", install_dir])
            .args(["--strip"])
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to install with cmake");
    }

    // die++
    println!("cargo:rustc-link-search=native={}", build_dir);
    println!("cargo:rustc-link-lib=static=die++");

    // die
    println!("cargo:rustc-link-search=native={}/dielib", lib_die_path);
    println!("cargo:rustc-link-lib=static=die");

    // die 3rd party
    println!(
        "cargo:rustc-link-search=native={}/XArchive/3rdparty/bzip2",
        lib_die_path
    );
    println!("cargo:rustc-link-lib=static=bzip2");
    println!(
        "cargo:rustc-link-search=native={}/XArchive/3rdparty/lzma",
        lib_die_path
    );
    println!("cargo:rustc-link-lib=static=lzma");
    println!(
        "cargo:rustc-link-search=native={}/XArchive/3rdparty/zlib",
        lib_die_path
    );
    println!("cargo:rustc-link-lib=static=zlib");

    println!("cargo:rustc-link-search=native={}/XCapstone", lib_die_path);
    println!("cargo:rustc-link-lib=static=capstone_x86");

    // qt
    if let Ok(qt_lib_path) = std::env::var("QT6_LIB_PATH") {
        // let lib_qt_path = "./libdie++/build/6.2.2/gcc_64/lib";
        println!("cargo:rustc-link-search=native={}", qt_lib_path);
    }

    println!("cargo:rustc-link-lib=dylib=Qt6Core");
    println!("cargo:rustc-link-lib=dylib=Qt6Qml");

    let target = std::env::var("TARGET").unwrap();

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    } else {
        // TODO (calladoum) other OSes
        unimplemented!();
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
}
