use crate::{Context, Error};
use poise::serenity_prelude::ReactionType;
use poise::{serenity_prelude::{self as serenity, Mentionable}, CreateReply};
use rand::Rng;
use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;

/// Ping the bot
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Flip a coin
///
/// Gives a 50-50 chance of heads or tails
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn coinflip(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let flip = {
        let mut rng = rand::rng();
        rng.random()
    };
    let message = if flip {
        "Heads"
    } else {
        "Tails"
    };

    ctx.say(message).await?;
    Ok(())
}

/// Crab party
// Taken from poise crate examples
#[poise::command(prefix_command, slash_command)]
pub async fn ferrisparty(ctx: Context<'_>) -> Result<(), Error> {
    let response = "```\n".to_owned()
        + &r"    _~^~^~_
\) /  o o  \ (/
  '_   ¬   _'
  | '-----' |
"
        .repeat(3)
        + "```";
    ctx.say(response).await?;
    Ok(())
}

/// Boop the bot!
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let uuid_boop = ctx.id();

    let reply = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(format!("{uuid_boop}"))
                .style(serenity::ButtonStyle::Primary)
                .label("Boop me!"),
        ])];

        CreateReply::default()
            .content("I want some boops!")
            .components(components)
    };

    ctx.send(reply).await?;

    let timeout = std::time::Duration::from_secs(120);
    let mut boop_count = 0;
    while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(timeout)
        .filter(move |mci| mci.data.custom_id == uuid_boop.to_string())
        .await
    {
        boop_count += 1;

        let mut msg = mci.message.clone();
        msg.edit(
            ctx,
            serenity::EditMessage::new().content(format!("Boop count: {boop_count}")),
        )
        .await?;

        mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
            .await?;
    }

    Ok(())
}

/// Mock a message
#[poise::command(prefix_command, slash_command, context_menu_command = "Mock")]
pub async fn mock(
    ctx: Context<'_>,
    #[description = "Message to mock"] msg: serenity::Message,
) -> Result<(), Error> {
    let response: String = {
        let mut rng = rand::rng();
        msg.content
            .chars()
            .map(|c| {
                // 40% chance of capitalizing letter, idk it feels better than 50-50
                if rng.random_bool(0.4) {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect()
    };

    ctx.say(&response).await?;

    Ok(())
}

/// Helper function to get a random roll value
fn die_roll(die: u32) -> u32 {
    let mut rng = rand::rng();
    rng.random_range(1..=die)
}

/// Roll some dice 
///
/// Enter a roll count and a die value optionally prefixed with  'd'
/// ```
/// !roll 5
/// !roll 5 d20
/// ```
#[poise::command(prefix_command, slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[min = 1]
    #[max = 100] // Min and max only apply to slash commands
    #[description = "Number of rolls to make"]
    roll_count: Option<u32>,
    #[description = "Dice type"]
    sides: Option<String>,
) -> Result<(), Error> {
    // Add a reaction if the command was a prefix command.
    // I don't think it is possible to do this with a slash command.
    if let poise::Context::Prefix(pctx) = ctx {
        pctx.msg.react(&pctx, ReactionType::Unicode("🎲".to_string())).await?;
    }

    // Default to 2 rolls
    let roll_count = roll_count.unwrap_or(2);

    let author = ctx.author().mention();

    // Since min and max only work with slash commands, have to bounds check here
    if !(1..=100).contains(&roll_count) {
        ctx.say(format!("Whoah {author}, your rolls are too powerful!")).await?;
        return Ok(())
    }

    // Parse specified dice value or go with d6 if failure / unspecified
    let dvalue: u32 = if let Some(dtype) = sides.clone() && dtype.starts_with('d') {
        dtype.split_once('d')
            .map(|pair| pair.1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(6)
    } else {
        sides
            .and_then(|s| s.parse().ok())
            .unwrap_or(6)
    };

    // Maybe I should change this to make it easier to read, I just like list folding
    let plurality = if roll_count > 1 { "'s" } else { "" };
    let (_, sum, message) = (0..roll_count)
        .fold((1, 0, format!("{author} rolled {roll_count} d{dvalue}{plurality}\n")),
        |(count, sum, msg), _| {
            let roll = die_roll(dvalue);
            let roll_info = if roll == 20 && dvalue == 20 {
                "- Critical Success! (20)"
            } else if roll == 1 && dvalue == 20 {
                "- Critical Failure! (1)"
            } else {
                ""
            };
            (count + 1, sum + roll, format!("{}\nRoll {}: {} {}", msg, count, roll, roll_info))
        }
    );

    ctx.say(format!("{}\n\nSum of all rolls: {}", message, sum)).await?;

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
                // Some articles return a title with an empty extract
                if extract.is_empty() {
                    String::from("That article exists but I couldn't get an extract!")
                }
                // If the message (and some formatting) is longer than 2000 chars,
                // truncate it
                else if title.len() + extract.len() + 6 > 2000 {
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
