fn main() {
    cpp_build::Config::new()
        .compiler("/usr/bin/g++")
        .flag("-frounding-math")
        .include("/usr/include/eigen3")
        .object("/usr/lib/libCGAL.so")
        .object("/usr/lib/libgmp.so")
        .object("/usr/lib/libmpfr.so")
        .build("src/lib.rs");

    println!("cargo:rustc-link-search=/usr/lib");
    println!("cargo:rustc-link-lib=CGAL");
    println!("cargo:rustc-link-lib=gmp");
    println!("cargo:rustc-link-lib=mpfr");
}
