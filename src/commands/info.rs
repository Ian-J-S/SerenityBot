use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, Mentionable};
use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;

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
#[poise::command(prefix_command, slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
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
#[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        ctx.say(response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
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
pub async fn joined(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let author = ctx.author_member().await.expect("Unable to retrieve command author");
    let joined = author.joined_at.expect("Unable to retrieve join time");
    let guild_name = ctx.guild().expect("Unable to retrieve guild").name.clone();
    ctx.say(format!("{} joined {}\n{}", author.mention(), guild_name, joined)).await?;
    Ok(())
}

/// Retrieve the closest matching wikipedia article
#[poise::command(prefix_command, slash_command)]
pub async fn wiki(
    ctx: Context<'_>,
    #[rest]
    #[description = "Article to search for"]
    request: String,
) -> Result<(), Error> {
    let req = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=1&explaintext=1&format=json&titles={}",
        request
    );

    let client = Client::new();
    let response: Value = client
        .get(req)
        .header(USER_AGENT, "rust-web-api-client")
        .send()
        .await?
        .json()
        .await?;

    let page = response["query"]["pages"]
        .as_object()
        .and_then(|pages| pages.values().next());

    let message = page.and_then(|page| {
        page["title"].as_str().and_then(|title| {
            page["extract"].as_str().map(|extract| {
                // If the message (and some formatting) is longer than 2000 chars,
                // truncate it
                if title.len() + extract.len() + 6 > 2000 {
                    format!("**{title}:**\n{}...", &extract[..(2000 - title.len() - 9)])
                } else {
                    format!("**{title}:**\n{}", &extract)
                }
            })
        })
    }).unwrap_or(String::from("Sorry, I couldn't find that article!"));

    ctx.say(message).await?;

    Ok(())
}
