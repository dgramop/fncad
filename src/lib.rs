use anyhow::Context;
use log::info;

mod dummy_solver;
mod renderer;
mod solve;

pub fn run() -> anyhow::Result<()> {
    info!("Opening window");
    let window = three_d::Window::new(three_d::WindowSettings {
        title: "fcnad".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .context("Failed to create window")?;

    let renderer = renderer::Renderer::new(&window);
    renderer.run(window)?;

    Ok(())
}
