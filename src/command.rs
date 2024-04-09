use anyhow::Error;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::{NoContext, Uuid};

pub async fn add(
    mut conn: &mut AsyncPgConnection,
    channel: String,
    command: String,
    value: String,
) -> anyhow::Result<()> {
    // remove command if it already exists
    // TODO: replace with UPDATE
    remove(conn, &channel, &command).await?;

    let ts = uuid::Timestamp::now(NoContext);
    diesel::insert_into(crate::schema::ttv_commands::table)
        .values(Command {
            id: uuid::Uuid::new_v7(ts),
            channel,
            command,
            value,
        })
        .execute(&mut conn)
        .await?;
    Ok(())
}

pub async fn remove(
    conn: &mut AsyncPgConnection,
    channel: &str,
    command: &str,
) -> anyhow::Result<()> {
    let (chnl, cmd) = (channel, command);
    {
        use crate::schema::ttv_commands::dsl::*;
        diesel::delete(
            ttv_commands
                .filter(channel.eq(chnl))
                .filter(command.eq(cmd)),
        )
        .execute(conn)
        .await?;
    }
    Ok(())
}

pub async fn get(
    conn: &mut AsyncPgConnection,
    channel: &str,
    command: &str,
) -> anyhow::Result<Command> {
    let (chnl, cmd) = (channel, command);
    {
        use crate::schema::ttv_commands::dsl::*;
        let mut cmds: Vec<Command> = ttv_commands
            .filter(channel.eq(chnl))
            .filter(command.eq(cmd))
            .limit(1)
            .select(Command::as_select())
            .load(conn)
            .await?;

        if cmds.is_empty() {
            Err(Error::msg("no commands found"))
        } else {
            Ok(cmds.remove(0))
        }
    }
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::ttv_commands)]
pub struct Command {
    id: Uuid,
    pub channel: String,
    pub command: String,
    pub value: String,
}
