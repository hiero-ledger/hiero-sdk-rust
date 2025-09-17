// Build script for hedera-proto-wasm
// This compiles ALL Hedera protobuf definitions using prost-build for WASM compatibility

use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> Result<()> {
    println!("cargo:warning=Building COMPLETE Hedera protobufs for WASM using prost-build");
    
    let proto_root = "../protobufs/services/hapi/hedera-protobuf-java-api/src/main/proto";
    
    if !Path::new(proto_root).exists() {
        anyhow::bail!("Proto root directory not found: {}. Make sure git submodules are initialized.", proto_root);
    }
    
    // Find all .proto files recursively
    let mut proto_files = Vec::new();
    for entry in WalkDir::new(proto_root).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("proto") {
            proto_files.push(entry.path().to_string_lossy().to_string());
        }
    }
    
    if proto_files.is_empty() {
        anyhow::bail!("No .proto files found in {}. Check your git submodules.", proto_root);
    }
    
    println!("cargo:warning=Found {} proto files", proto_files.len());
    
    // Brief summary of found files
    if proto_files.len() > 5 {
        println!("cargo:warning=Including {} proto files (showing first 5):", proto_files.len());
        for (i, file) in proto_files.iter().take(5).enumerate() {
            println!("cargo:warning=  {}: {}", i + 1, file.split('/').last().unwrap_or(file));
        }
    } else {
        for file in &proto_files {
            println!("cargo:warning=Including: {}", file.split('/').last().unwrap_or(file));
        }
    }
    
    // Configure prost-build
    let mut config = prost_build::Config::new();
    
    // Set output file
    config.include_file("hedera_protos.rs");
    
    // Configure prost for clean protobuf generation
    
    // Compile all proto files
    config.compile_protos(&proto_files, &[proto_root])?;
    
    println!("cargo:warning=Successfully compiled {} protobuf files for WASM", proto_files.len());
    
    // Tell cargo to rerun if proto files change
    println!("cargo:rerun-if-changed={}", proto_root);
    
    Ok(())
} 