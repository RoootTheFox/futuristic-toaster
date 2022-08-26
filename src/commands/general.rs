use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn twitch(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "https://twitch.tv/notbunder").await?;

    Ok(())
}

#[command]
pub async fn youtube(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "https://www.youtube.com/channel/UCe_onN5fgpoTllOoaBvODeg").await?;

    Ok(())
}

#[command]
pub async fn onlyfans(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "you sick fuck").await?;

    Ok(())
}