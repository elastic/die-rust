// build.rs
extern crate cmake;

fn main() {
    // cxx_build::bridge("src/lib.rs")
    //     .std("c++20")
    //     .file("src/die.cpp")
    //     .compile("die-rust");

    let target = cmake::Config::new("libdie++").build();
    println!("cargo:rustc-link-search=native={}", target.display());
    println!("cargo:rustc-link-lib=static=die++");

    let target = std::env::var("TARGET").unwrap();

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else {
        unimplemented!();
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/die.cpp");
    println!("cargo:rerun-if-changed=inc/die.hpp");
}
