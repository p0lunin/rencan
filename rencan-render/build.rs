use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=shaders/include/defs.glsl");
    println!("cargo:rerun-if-changed=shaders/include/ray_tracing.glsl");
    println!("cargo:rerun-if-changed=shaders/ray_tracing.glsl");
    println!("cargo:rerun-if-changed=shaders/checkboard_pattern.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning.glsl");
    println!("cargo:rerun-if-changed=shaders/squeeze.glsl");
    println!("cargo:rerun-if-changed=shaders/sky/blue.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning/lightning.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning/make_lightning_rays_lambertarian.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning/trace_rays_to_lights.glsl");
    println!("cargo:rerun-if-changed=shaders/lightning/copy_from_buffer_to_image.glsl");
    Ok(())
}
