use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun if config changes
    println!("cargo:rerun-if-changed=config/default.toml");
    
    // Get target architecture info
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-cfg=target=\"{}\"", target);
    
    // Create assets directory if it doesn't exist
    let out_dir = env::var("OUT_DIR").unwrap();
    let assets_dir = Path::new(&out_dir).join("../../../assets");
    fs::create_dir_all(&assets_dir).ok();
    
    println!("cargo:warning=Building for target: {}", target);
}
