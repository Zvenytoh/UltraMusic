//! Embeds the application icon and package metadata in Windows executables.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=assets/ultramusic.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon_with_id("assets/ultramusic.ico", "ULTRAMUSIC_ICON")
            .set("ProductName", "UltraMusic")
            .set("FileDescription", "UltraMusic background music player")
            .compile()?;
    }
    Ok(())
}
