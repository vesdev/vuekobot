use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection};
use tmi::Badge;
use tracing::error;

use crate::config::Config;

pub struct VuekoTwitch {}

impl VuekoTwitch {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn serve(
        &mut self,
        config: &Config,
        pgpool: &Pool<AsyncPgConnection>,
    ) -> eyre::Result<()> {
        let mut client = tmi::Client::builder()
            .credentials(tmi::Credentials {
                login: config.user.clone(),
                token: Some(config.tmi_password.clone()),
            })
            .connect()
            .await?;
        client.join_all(config.channels.clone()).await?;
        // let mut ctx = Context { client, &pgpool };
        loop {
            let Ok(msg) = client.recv().await else {
                error!("Failed to receive twitch irc message");
                continue;
            };
            if let Err(e) = Self::map_command(&mut client, pgpool, config, msg).await {
                error!("{e}");
            }
        }
    }

    async fn map_command(
        client: &mut tmi::Client,
        pgpool: &Pool<AsyncPgConnection>,
        config: &Config,
        msg: tmi::IrcMessage,
    ) -> eyre::Result<()> {
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

                            let mut conn = pgpool.get().await?;
                            // TODO: handle errors properly
                            match args.next() {
                                Some("add") => {
                                    let _ =
                                        db::add(client, &mut conn, msg.channel().into(), &mut args)
                                            .await;
                                }
                                Some("remove") => {
                                    let _ = db::remove(
                                        client,
                                        &mut conn,
                                        msg.channel().into(),
                                        &mut args,
                                    )
                                    .await;
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
                let mut conn = pgpool.get().await?;
                let Ok(command) = crate::command::get(&mut conn, msg.channel(), cmd).await else {
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
}

mod db {
    use std::str::SplitWhitespace;

    use diesel_async::AsyncPgConnection;

    use crate::command;
    async fn reply_error(client: &mut tmi::Client, channel: &str, msg: &str) {
        let _ = client.privmsg(channel, msg).send().await;
    }

    pub async fn add(
        client: &mut tmi::Client,
        conn: &mut AsyncPgConnection,
        channel: String,
        args: &mut SplitWhitespace<'_>,
    ) -> eyre::Result<()> {
        let (Some(command), Some(value)) = (
            args.next(),
            args.map(|s| s.to_string()).reduce(|a, b| a + " " + &b),
        ) else {
            reply_error(
                client,
                &channel,
                "Command missing arguments. Usage: !v command add <name> <value>",
            )
            .await;
            return Ok(());
        };

        if command::add(conn, channel.clone(), command.into(), value)
            .await
            .is_err()
        {
            reply_error(
                client,
                &channel,
                &format!("Failed to add command {}", command),
            )
            .await;
        }
        Ok(())
    }
    pub async fn remove(
        client: &mut tmi::Client,
        conn: &mut AsyncPgConnection,
        channel: String,
        args: &mut SplitWhitespace<'_>,
    ) -> eyre::Result<()> {
        let Some(command) = args.next() else {
            reply_error(
                client,
                &channel,
                "Command missing arguments. Usage: !v command remove <name>",
            )
            .await;
            return Ok(());
        };

        if command::remove(conn, &channel, command).await.is_err() {
            reply_error(
                client,
                &channel,
                &format!("Failed to remove command {}", command),
            )
            .await;
        }
        Ok(())
    }
}
