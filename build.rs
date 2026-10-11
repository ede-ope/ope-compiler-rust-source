fn main() {
    // Tell Cargo where to find the .lib file
    // "." means current directory (project root)
    // println!("cargo:rustc-link-search=native=.");

    // Tell Cargo which lib to link
    // Use the filename without the .lib extension
    // println!("cargo:rustc-link-lib=static=LLVM-C");

    // LLVM requires these Windows system libraries
    // println!("cargo:rustc-link-lib=shell32");
    // println!("cargo:rustc-link-lib=ole32");
    // println!("cargo:rustc-link-lib=uuid");
    // println!("cargo:rustc-link-lib=advapi32");

    println!(r"cargo:rustc-link-search=native=C:\LLVM\lib");
    
    // if LLVM-C.lib is MSVC format, use dylib not static
    println!("cargo:rustc-link-lib=dylib=LLVM-C");
    
    // also add the bin dir so the .dll is found at runtime
    println!(r"cargo:rustc-link-search=native=C:\LLVM\bin");

    let host = std::env::var("HOST").unwrap();
    println!("cargo:rustc-env=BUILD_HOST={host}");
}
