use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use diesel::Connection;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::{api::VuekoApi, config::Config, twitch::VuekoTwitch};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct App {
    addr: SocketAddr,
    config: Config,
}

impl App {
    pub async fn new(addr: &str, config: Config) -> eyre::Result<Self> {
        let addr = SocketAddr::new(IpAddr::from_str(addr)?, 45861);
        Ok(Self { addr, config })
    }

    pub async fn serve(&mut self) -> eyre::Result<()> {
        let dburl = self.config.database.clone();

        tokio::task::spawn_blocking(move || -> eyre::Result<()> {
            let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&dburl)?;
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|_| eyre::Error::msg("Unable to run database migrations"))?;
            Ok(())
        });

        let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&self.config.database);
        let pgpool = Pool::builder().build(mgr).await?;

        {
            let mut api = VuekoApi::new(self.addr).await;
            let pgpool = pgpool.clone();
            let config = self.config.clone();
            tokio::spawn(async move {
                // TODO: handle api errors
                let _ = api.serve(&config, pgpool).await;
            });
        }

        let mut ttv = VuekoTwitch::new();
        ttv.serve(&self.config, &pgpool).await?;

        Ok(())
    }
}
