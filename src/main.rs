use std::env;

use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use serde::Deserialize;
use tmi::Badge;

mod command;
mod schema;

#[derive(Deserialize)]
struct Config {
    channels: Vec<String>,
    database: String,
    tmi_password: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let config: Config = toml::from_str(&std::fs::read_to_string(args[1].clone())?)?;

    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database);
    let pool = Pool::builder().build(mgr).await?;
    let mut client = tmi::Client::builder()
        .credentials(tmi::Credentials {
            login: "vuekobot".into(),
            token: Some(config.tmi_password),
        })
        .connect()
        .await?;
    client.join_all(config.channels.clone()).await?;

    loop {
        let msg = client.recv().await?;
        match msg.as_typed()? {
            tmi::Message::Privmsg(msg) => {
                println!("{msg:?}");
                let Some(args) = msg.text().strip_prefix('!') else {
                    continue;
                };
                let mut args = args.split_whitespace();

                let mut conn = pool.get().await?;
                let Some(cmd) = args.next() else {
                    continue;
                };

                let is_privileged = msg
                    .badges()
                    .find(|b| matches!(b, Badge::Broadcaster | Badge::Moderator));

                // vueko commands
                if is_privileged.is_some() && cmd == "v" && args.next() == Some("command") {
                    match args.next() {
                        Some("add") => {
                            let (Some(command), Some(value)) = (
                                args.next(),
                                args.map(|s| s.to_string()).reduce(|a, b| a + " " + &b),
                            ) else {
                                continue;
                            };

                            if command::add(&mut conn, msg.channel().into(), command.into(), value)
                                .await
                                .is_err()
                            {
                                client
                                    .privmsg(
                                        msg.channel(),
                                        &format!("Failed to add command {}", command),
                                    )
                                    .send()
                                    .await?;
                            }
                        }
                        Some("remove") => {
                            let Some(command) = args.next() else {
                                continue;
                            };

                            if command::remove(&mut conn, msg.channel(), command)
                                .await
                                .is_err()
                            {
                                client
                                    .privmsg(
                                        msg.channel(),
                                        &format!("Failed to remove command {}", command),
                                    )
                                    .send()
                                    .await?;
                            }
                        }
                        Some(&_) => continue,
                        None => continue,
                    };

                    continue;
                }
                println!("a");

                // otherwise we can assume its a user defined command
                let command = command::get(&mut conn, msg.channel(), cmd).await?;
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
    }
}
