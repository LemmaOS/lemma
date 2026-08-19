fn main() -> Result<(), Box<dyn std::error::Error>> {
    connectrpc_build::Config::new()
        .files(&["../../proto/lemma/v1/auth.proto"])
        .includes(&["../../proto"])
        .include_file("_connectrpc.rs")
        .compile()?;
    Ok(())
}
