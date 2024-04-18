use std::str::FromStr;

use fofo_utils::{
    config::Config,
    meta::{Commands, MetaInfo},
};
use shared_core::SharedCore;
use tracing::info;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::main]
async fn main() {
    let filtered_layer = fmt::Layer::default().with_filter(filter::LevelFilter::WARN);
    let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
    tracing_subscriber::registry().with(filtered_layer).init();

    let meta = MetaInfo::new();
    let config_path = if cfg!(debug_assertions) {
        meta.data_path.join("config.dev.toml")
    } else {
        meta.data_path.join("config.toml")
    };
    let config = Config::new(config_path).await.unwrap();
    info!("Using config:\n {:?}", config);
    reload_handle
        .modify(|layer| {
            *layer.filter_mut() = filter::LevelFilter::from_str(&config.log_level)
                .unwrap_or(filter::LevelFilter::INFO)
        })
        .unwrap();

    let core = SharedCore::new(config, meta.clone()).await;

    match core.get_meta().cmd {
        Commands::Serve => start(core).await,
    }
}

async fn start(core: SharedCore) {
    let meta = core.get_meta();
    fofo_server::run(core, &meta.host, meta.port).await.unwrap();
}
