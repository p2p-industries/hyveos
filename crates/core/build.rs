fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .compile(&["../../protos/script.proto"], &["../../protos"])?;
    Ok(())
}
