use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    println!("flareTUI - Cloudflare Management Terminal");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
