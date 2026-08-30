fn main() -> Result<(), Box<dyn std::error::Error>> {
    connectrpc_build::Config::new()
        .files(&[
            "../../proto/lemma/v1/auth.proto",
            "../../proto/lemma/v1/errors.proto",
            "../../proto/lemma/v1/provider.proto",
            "../../proto/lemma/v1/conversation.proto",
            "../../proto/lemma/v1/chat.proto",
            "../../proto/lemma/v1/sync.proto",
            "../../proto/lemma/v1/storage.proto",
        ])
        .includes(&["../../proto"])
        .include_file("_connectrpc.rs")
        .compile()?;
    Ok(())
}
