use offchain::error::*;
use offchain::logic::descartes_logic::{main_loop, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = todo!();

    main_loop(&config).await
}
