use offchain::config::ApplicationConfig;
use offchain::error::*;
use offchain::logic::rollups_logic::main_loop;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ApplicationConfig::initialize()?;
    main_loop(&config).await
}