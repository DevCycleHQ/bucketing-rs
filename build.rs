fn main() {
    #[cfg(feature = "protobuf")]
    {
        prost_build::compile_protos(&["proto/config.proto"], &["proto/"]).unwrap();
    }
}
