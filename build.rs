fn main() {
    let builder = tonic_build::configure();
    builder
        .compile(&["proto/model.proto", "proto/service.proto"], &["proto/"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e))
}
