use std::env;

use app::App;
use eyre::Context;

mod api;
mod app;
mod command;
mod config;
mod schema;
mod twitch;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1).unwrap_or(&"../vueko.toml".into()).clone();

    let config =
        config::from_path(config_path.into()).context("Unable to parse configuration from path")?;

    let mut app = App::new("127.0.0.1", config).await?;
    app.serve().await
}
