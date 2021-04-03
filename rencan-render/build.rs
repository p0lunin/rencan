use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=shaders/include/defs.glsl");
    println!("cargo:rerun-if-changed=shaders/include/ray_tracing.glsl");
    println!("cargo:rerun-if-changed=shaders/show_xyz_ordinates.glsl");
    println!("cargo:rerun-if-changed=shaders/ray_tracing.glsl");
    println!("cargo:rerun-if-changed=shaders/checkboard_pattern.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning.glsl");
    println!("cargo:rerun-if-changed=shaders/squeeze.glsl");
    println!("cargo:rerun-if-changed=shaders/sky/blue.glsl");
    Ok(())
}
