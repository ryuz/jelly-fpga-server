fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("./protos/jelly_fpga_control.proto")?;
    Ok(())
}
