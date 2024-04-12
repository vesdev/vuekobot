use std::net::SocketAddr;

use axum::{
    extract::{Path, State},
    Json, Router,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection};
use serde::Serialize;

use crate::{config::Config, schema::ttv_commands};

#[derive(Clone)]
struct ApiState {
    pgpool: Pool<AsyncPgConnection>,
}

pub struct VuekoApi {
    addr: SocketAddr,
}

impl VuekoApi {
    pub async fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn serve(
        &mut self,
        config: &Config,
        pgpool: Pool<AsyncPgConnection>,
    ) -> eyre::Result<()> {
        let state = ApiState { pgpool };
        let app = Router::new()
            .route(
                "/api/v1/channel/:channel/commands.json",
                axum::routing::get(list_commands),
            )
            .with_state(state);
        let listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();
        axum::serve(listener, app.into_make_service()).await?;
        Ok(())
    }
}

#[derive(Serialize)]
struct RespCommand {
    id: String,
    channel: String,
    command: String,
    value: String,
}

#[derive(Serialize)]
struct RespCommands {
    commands: Vec<RespCommand>,
}

async fn list_commands(
    Path(channel): Path<String>,
    State(state): State<ApiState>,
) -> Json<RespCommands> {
    use crate::command::Command;
    use diesel_async::RunQueryDsl;

    let Ok(mut conn) = state.pgpool.get().await else {
        return RespCommands { commands: vec![] }.into();
    };

    let Ok(mut commands) = crate::schema::ttv_commands::table
        .filter(ttv_commands::dsl::channel.eq(format!("#{channel}")))
        .select(Command::as_select())
        .limit(10)
        .load::<Command>(&mut conn)
        .await
    else {
        return RespCommands { commands: vec![] }.into();
    };

    RespCommands {
        commands: commands
            .drain(..)
            .map(|c| RespCommand {
                id: c.id.to_string(),
                channel: c.channel,
                command: c.command,
                value: c.value,
            })
            .collect(),
    }
    .into()
}
