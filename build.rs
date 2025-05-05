fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=assets/");
    
    // You can add additional build steps here if needed
    // For example, compiling resources or preparing assets
}