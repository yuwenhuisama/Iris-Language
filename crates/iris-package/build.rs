fn main() -> Result<(), std::env::VarError> {
    let target = std::env::var("TARGET")?;
    println!("cargo:rustc-env=IRIS_PACKAGE_TARGET={target}");
    Ok(())
}
