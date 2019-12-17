fn main() {
    cpp_build::Config::new()
        .compiler("/usr/bin/g++")
        .flag("-frounding-math")
        .include("/usr/include/eigen3")
        .object("/usr/lib/libCGAL.so")
        .object("/usr/lib/libgmp.so")
        .object("/usr/lib/libmpfr.so")
        .build("src/lib.rs");
}
