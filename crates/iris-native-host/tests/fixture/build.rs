use sha2::{Digest, Sha256};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let digest: [u8; 32] = Sha256::digest(std::fs::read("module.json")?).into();
    println!("cargo:rerun-if-changed=module.json");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR missing")?);
    std::fs::write(
        output.join("metadata_digest.rs"),
        format!("const METADATA_SHA256: [u8; 32] = {digest:?};"),
    )?;
    Ok(())
}
