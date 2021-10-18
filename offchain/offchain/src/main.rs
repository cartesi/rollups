use offchain::error::*;
use offchain::logic::descartes_logic::{main_loop, Config};

use block_subscriber::config::BSConfig;
use tx_manager::config::TMConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let logic_config: Config = todo!();
    let tm_config: TMConfig = todo!();
    let bs_config: BSConfig = todo!();

    main_loop(&logic_config, &tm_config, &bs_config).await
}
