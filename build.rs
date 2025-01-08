// build.rs
// https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    let base_dir = "./libdie++";
    let build_dir = "./libdie++/build";
    let lib_die_path = "./libdie++/build/_deps/dielibrary-build/src";

    #[cfg(debug_assertions)]
    let build_type = "Debug";
    #[cfg(not(debug_assertions))]
    let build_type = "Release";

    // CMake configure
    {
        assert!(std::process::Command::new("cmake")
            .current_dir(base_dir)
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
            .args(["--config", build_type])
            .current_dir(base_dir)
            .spawn()
            .unwrap()
            .wait()
            .expect("failed to build with cmake")
            .success());
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
        let lib_qt_path = "./libdie++/build/6.2.2/msvc2019_64/lib";
        println!("cargo:rustc-link-search=native={}", lib_qt_path);

        println!(
            "cargo:rustc-link-search=native={}/{}",
            build_dir, build_type
        );
        println!(
            "cargo:rustc-link-search=native={}/_deps/dielibrary-build/src/dielib/{}",
            build_dir, build_type
        );
        for _mod in ["bzip2", "lzma", "zlib"].iter() {
            println!(
                "cargo:rustc-link-search=native={}/XArchive/3rdparty/{}/{}",
                lib_die_path, _mod, build_type
            );
        }

        println!(
            "cargo:rustc-link-search=native={}/XCapstone/{}",
            lib_die_path, build_type
        );

        println!("cargo:rustc-link-lib=dylib=Crypt32");
        println!("cargo:rustc-link-lib=dylib=Wintrust");
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
}
