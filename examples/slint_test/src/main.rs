use anyhow::Result;
use slint::{include_modules};

include_modules!();

fn main() -> Result<()> {
    println!("Starting Slint test with DRM/KMS backend");

    let ui = MainWindow::new()?;
    
    ui.show()?;
    
    let window = ui.window();
    println!("Window size: {}x{}", window.size().width, window.size().height);
    
    ui.run()?;

    Ok(())
}