use std::env;

use diesel::{Connection};
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::Deserialize;
use tmi::Badge;
use tracing::error;

mod command;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Deserialize)]
struct Config {
    user: String,
    tmi_password: String,
    channels: Vec<String>,
    database: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();

    let config: Config = toml::from_str(&std::fs::read_to_string(
        args.get(1).unwrap_or(&"./vuekobot.toml".into()).clone(),
    )?)?;

    let dburl = config.database.clone();
    tokio::task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&dburl)?;
            conn.run_pending_migrations(MIGRATIONS)?;
            Ok(())
        },
    );

    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database);
    let pgpool = Pool::builder().build(mgr).await?;

    let mut client = tmi::Client::builder()
        .credentials(tmi::Credentials {
            login: config.user.clone(),
            token: Some(config.tmi_password.clone()),
        })
        .connect()
        .await?;
    client.join_all(config.channels.clone()).await?;

    loop {
        let msg = client.recv().await?;
        let mut conn = pgpool.get().await?;
        if let Err(e) = handle_commands(&config, &mut conn, &mut client, msg).await {
            error!("{e}");
        }
    }
}

async fn handle_commands(
    config: &Config,
    conn: &mut AsyncPgConnection,
    client: &mut tmi::Client,
    msg: tmi::IrcMessage,
) -> anyhow::Result<()> {
    match msg.as_typed()? {
        tmi::Message::Privmsg(msg) => {
            let Some(args) = msg.text().strip_prefix('!') else {
                return Ok(());
            };

            let mut args = args.split_whitespace();

            let Some(cmd) = args.next() else {
                return Ok(());
            };

            let is_privileged = msg
                .badges()
                .find(|b| matches!(b, Badge::Broadcaster | Badge::Moderator));

            // vueko commands
            if cmd == "v" {
                match args.next() {
                    Some("command") => {
                        if is_privileged.is_none() {
                            return Ok(());
                        }
                        match args.next() {
                            Some("add") => {
                                let (Some(command), Some(value)) = (
                                    args.next(),
                                    args.map(|s| s.to_string()).reduce(|a, b| a + " " + &b),
                                ) else {
                                    reply_error(
                                        client,
                                        msg.channel(),
                                        "Command missing arguments. Usage: !v command add <name> <value>",
                                    )
                                    .await;
                                    return Ok(());
                                };

                                if command::add(conn, msg.channel().into(), command.into(), value)
                                    .await
                                    .is_err()
                                {
                                    reply_error(
                                        client,
                                        msg.channel(),
                                        &format!("Failed to add command {}", command),
                                    )
                                    .await;
                                }
                            }
                            Some("remove") => {
                                let Some(command) = args.next() else {
                                    reply_error(
                                        client,
                                        msg.channel(),
                                        "Command missing arguments. Usage: !v command remove <name>",
                                    )
                                    .await;
                                    return Ok(());
                                };

                                if command::remove(conn, msg.channel(), command).await.is_err() {
                                    reply_error(
                                        client,
                                        msg.channel(),
                                        &format!("Failed to remove command {}", command),
                                    )
                                    .await;
                                }
                            }
                            Some(&_) => return Ok(()),
                            None => return Ok(()),
                        };
                    }
                    Some("ping") => {
                        client.privmsg(msg.channel(), "pong").send().await?;
                    }
                    Some(&_) => return Ok(()),
                    None => return Ok(()),
                }

                return Ok(());
            }

            // otherwise we can assume its a user defined command
            let Ok(command) = crate::command::get(conn, msg.channel(), cmd).await else {
                return Ok(());
            };

            client.privmsg(msg.channel(), &command.value).send().await?;
        }
        tmi::Message::Reconnect => {
            client.reconnect().await?;
            client.join_all(config.channels.clone()).await?;
        }
        tmi::Message::Ping(ping) => {
            client.pong(&ping).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn reply_error(client: &mut tmi::Client, channel: &str, msg: &str) {
    let _ = client.privmsg(channel, msg).send().await;
}
