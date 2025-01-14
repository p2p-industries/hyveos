use std::{env::var, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos_path = Path::new(&var("CARGO_MANIFEST_DIR").unwrap()).join("protos");
    tonic_build::configure()
        .build_client(true)
        .compile_protos(&[protos_path.join("script.proto")], &[protos_path])?;
    Ok(())
}
