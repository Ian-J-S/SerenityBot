use crate::{Context, Error, utils::get_last_message};
use poise::serenity_prelude::{self as serenity, Mentionable};
use std::process::Command;

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Shitty bot, adapted from examples",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Shut down the bot. Owners only.
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

/// Vote for something
///
/// Enter `!vote pumpkin` to vote for pumpkins
// Taken from poise crate examples
#[poise::command(prefix_command, slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().await;
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
    ctx.say(response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// !getvotes
/// !getvotes pumpkin
/// ```
// Taken from poise crate examples
#[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().await.get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        ctx.say(response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().await.iter() {
            response += &format!("{}: {} votes", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        ctx.say(response).await?;
    };

    Ok(())
}

/// Echo content of a message
// Taken from poise crate examples
#[poise::command(prefix_command, context_menu_command = "Echo", slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    ctx.say(&msg.content).await?;
    Ok(())
}

/// Tells you when you joined the server in UTC
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn joined(ctx: Context<'_>) -> Result<(), Error> {
    let author = ctx.author_member().await.expect("Unable to retrieve command author");
    let joined = author.joined_at.expect("Unable to retrieve join time");
    let guild_name = ctx.guild().expect("Unable to retrieve guild").name.clone();
    ctx.say(format!("{} joined {}\n{}", author.mention(), guild_name, joined)).await?;
    Ok(())
}

/// Helper function for printing an 's' with time values
fn plural(n: u64) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Helper function to get the uptime of the server running the bot.
fn get_server_uptime() -> Result<String, Error> {
    let com = Command::new("uptime")
        .arg("-p")
        .output()?;

    let output = String::from_utf8(com.stdout)?;

    Ok(output)
}

/// Gives uptime of bot and server running the bot.
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    // Bot
    let duration = ctx.data().start_time.elapsed();
    let seconds = duration.as_secs();
    
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    // Server
    let server_uptime = get_server_uptime()?;
    
    ctx.say(format!(
        "Bot has been up {} day{}, {} hour{}, {} minute{}, {} second{}\n\
        Server has been {}",
        days, plural(days),
        hours, plural(hours),
        minutes, plural(minutes),
        secs, plural(secs),
        server_uptime,
    )).await?;
    
    Ok(())
}

/// Because of course we need a prse command
#[poise::command(prefix_command, slash_command)]
pub async fn prse(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("PReSEnting: https://github.com/Asterisk007/prse\n[This programming language is not endorsed by the University, nor this Discord server.]").await?;
    Ok(())
}

/// Removes a message that the bot had sent
#[poise::command(prefix_command, slash_command,
required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS")]
pub async fn private(
    ctx: Context<'_>,
    #[description = "Message to delete (gets last message if not provided)"]
    msg: Option<serenity::Message>,
) -> Result<(), Error> {
    let res = match msg {
        Some(msg) => {
            msg.delete(&ctx).await
        }
        None => {
            let msg = get_last_message(&ctx).await?;
            msg.delete(&ctx).await
        }
    };

    if let Err(e) = res {
        ctx.say(format!("Unable to delete command: {e}")).await?;
    }

    Ok(())
}
