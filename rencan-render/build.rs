use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=shaders/compute_rays.glsl");
    println!("cargo:rerun-if-changed=shaders/show_xyz_ordinates.glsl");

    Ok(())
}
