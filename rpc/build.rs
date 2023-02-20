fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("financial-analysis-grpc/api.proto")?;
    Ok(())
}
