use crate::{Context, Error};
use chrono::Datelike;
use poise::serenity_prelude::{GetMessages, Mention, ReactionType};
use poise::{serenity_prelude::{self as serenity, Mentionable}, CreateReply};
use rand::{Rng, seq::IndexedRandom};
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
    let message = ["Heads", "Tails"].choose(&mut rand::rng())
        .unwrap(); // Only returns 'none' if array is empty

    ctx.say(*message).await?;
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
// Taken from poise crate examples
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

fn mock_helper(msg: &str) -> String {
    let mut rng = rand::rng();
    msg.chars()
        .map(|c| {
            // 40% chance of capitalizing letter, idk it feels better than 50-50
            if rng.random_bool(0.4) {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

/// Mock a message
#[poise::command(prefix_command, slash_command)]
pub async fn mock(
    ctx: Context<'_>,
    #[description = "Message to mock"] 
    msg: Option<serenity::Message>,
) -> Result<(), Error> {
    let msg = match msg {
        Some(msg) => msg,
        None => get_last_message(&ctx).await?,
    };

    let response = mock_helper(&msg.content);

    ctx.say(response).await?;

    Ok(())
}

/// Context menu version of mock?
#[poise::command(context_menu_command = "Mock")]
pub async fn mock_ctx_menu(
    ctx:Context<'_>,
    #[description = "Message to mock"] 
    msg: serenity::Message,
) -> Result<(), Error> {
    let response = mock_helper(&msg.content);
    ctx.say(response).await?;
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
    let dvalue: u32 = match sides.clone() {
        Some(dtype) if dtype.starts_with('d') => {
            dtype.split_once('d')
                .map(|(_, n)| n)
                .and_then(|s| s.parse().ok())
                .unwrap_or(6)
        }
        Some(s) => s.parse().ok().unwrap_or(6),
        None => 6,
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

/// Bot says what you put.
#[poise::command(prefix_command, slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[rest]
    #[description = "Message to say"]
    msg: String,
) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

/// Helper function to chooose a random 'ban' message
fn choose_ban_msg(user_mentioned: Mention) -> String {
    let ban_messages = [
        format!("brb, banning {user_mentioned}."),
        format!("you got it, banning {user_mentioned}."),
        format!("{user_mentioned}, you must pay for your crimes. A ban shall suffice."),
        format!("today's controvesial opinion reward goes to {user_mentioned}. The prize? A ban, duh."),
        format!("{user_mentioned} gotta ban you now. Sorry."),
        format!("{user_mentioned} stop talking before you--oh, wait. Too late."),
        format!("{user_mentioned}, really? I wish I could ban you more than once."),
        format!("{user_mentioned} the game of hide and seek is over, tag, you're banned."),
        String::from("Banned: the server has automatically banned you for saying a bad word."),
    ];

    let ban_easter_eggs = [
        format!("{user_mentioned} I WARNED YOU ABOUT STAIRS BRO. I TOLD YOU."),
        format!("Let's be honest with ourselves: we just wanted to ping {user_mentioned} twice."),
        format!("{user_mentioned} has broken the unspoken rule."),
    ];

    let mut rng = rand::rng();

    if rng.random_bool(0.1) {
        ban_easter_eggs.choose(&mut rng)
            .unwrap_or(&ban_easter_eggs[0]).to_string()
    } else {
        ban_messages.choose(&mut rng)
            .unwrap_or(&ban_messages[0]).to_string()
    }
}

/// Helper function to get the author of the last message in the current channel
async fn get_last_message(ctx: &Context<'_>) -> Result<serenity::Message, Error> {
    let channel = ctx.channel_id();

    // If prefix command is used, it counts as the last message, so we need 2 messages.
    // Otherwise, just 1 message is needed.
    let limit = if let poise::Context::Prefix(_) = ctx {
        2
    } else {
        1
    };

    let messages = channel
        // Limit to 2 (not 1) or else the ban command itself is chosen
        .messages(ctx, GetMessages::new().limit(limit))
        .await?;

    let message = messages.last()
        .ok_or("Unable to get last message")?;

    Ok(message.clone())
}

/// Bans (but not actually) the person mentioned
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "@ user to 'ban'"]
    user: Option<Mention>,
) -> Result<(), Error> {
    let mention = match user {
        Some(user) => user,
        None => {
            let msg = get_last_message(&ctx).await?;
            msg.author.mention()
        }
    };

    ctx.say(choose_ban_msg(mention)).await?;

    Ok(())
}

/// YEET
#[poise::command(prefix_command, slash_command)]
pub async fn yeet(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!("{} YEET!\nhttps://youtu.be/mbDkgGv-vJ4?t=4", ctx.author().mention())).await?;
    Ok(())
}

/// Happy Halloween! Returns trick or treat, only works during spooktober!
#[poise::command(prefix_command, slash_command)]
pub async fn trickortreat(ctx: Context<'_>) -> Result<(), Error> {
    let month = chrono::Local::now().month();

    let message = if month == 10 {
        let choices = ["Trick", "Treat"];
        choices.choose(&mut rand::rng()).copied()
    } else {
        let choices = [
            "it is not spooktober, try again later",
            "try again in october",
             "YOU DARE TRY TO TRICK OR TREAT WHEN IT'S NOT OCTOBER"
        ];
        choices.choose(&mut rand::rng()).copied()
    }.ok_or("error in choosing random message")?;

    ctx.say(message).await?;

    Ok(())
}

fn owofy(input: &str) -> String {
    let uwu_suffixes = [
        " (´・ω・｀)",
        " (๑•́ ₃ •̀๑)",
        " (• o •)",
        " (⁎˃ᆺ˂)",
        " (╯﹏╰）",
        " (●´ω｀●)",
        " (◠‿◠✿)",
        " (✿ ♡‿♡)",
        r" (❁´◡\`❁)",
        r" (　\'◟ \')",
        " (；ω；)",
        r" (´･ω･\`)",
        " o3o",
        " :3",
        " :D",
        " :P",
        r" ;\_;",
        " <{^v^}>",
        r" >\_<",
        " UwU",
        " ^-^",
        " xD",
        " ÙωÙ",
        " ㅇㅅㅇ",
        " （＾ｖ＾）",
        r" \*starts howling\*",
        r" \*leaps up and down\*",
        r" \*wags tail\*",
    ];

    let uwu_substitutions = [
        ("r", "w"),
        ("l", "w"),
        ("the ", "da "),
        ("th", "d"),
        ("hi", "hai"),
        ("has", "haz"),
        ("have", "haz"),
        ("is", "iws"),

        // some words have already been uwu-ized
        ("fuck", "henck"),
        ("bitch", "vewwy nice lady"),
        ("shait", "poot"),
        (" ass ", " fwuffey tail "),
        ("kill", "nuzzle"),
        ("god", "sonic"),
        ("jesus christ", "cheese and wice"),
        ("degenewates", "cutie pies"),
        ("degenewate", "cutie pie"),
        ("diwsgusting", "bulgy wulgy"),
        ("gwossest", "bulgiest"),
        ("gwoss", "AMAZEBALLS (✿ ♡‿♡)"),
        ("nasty", "musky"),
        ("hand", "paw"),

        // compounding uwu-ness
        ("uwu", "uwuwuwu"),
        ("owo", "owowowo"),
        ("you ", "uwu "),
        ("dude", "duwude"),
        ("to", "towo"),
        ("no", "nowo"),
        ("oh", "owo"),
        ("do ", "dowo "),
    ];

    let mut output = String::from(input);
    for (from, to) in uwu_substitutions {
        output = output.replace(from, to);
    }

    let mut rng = rand::rng();
    let mut final_out = String::with_capacity(output.len());

    for ch in output.chars() {
        if matches!(ch, '.' | '!' | '?') && rng.random_bool(0.2) {
            let suffix = uwu_suffixes.choose(&mut rng).unwrap();
            final_out.push_str(suffix);
            final_out.push(' ');
        } else {
            final_out.push(ch);
        }
    }

    final_out
}

/// Convewts da specified stwing into OwO speak
///
/// If no argument is provided, use last message.
#[poise::command(prefix_command, slash_command)]
pub async fn owo(
    ctx:Context<'_>,
    msg: Option<serenity::Message>,
) -> Result<(), Error> {
    // If no argument provided, get last message
    let msg = match msg {
        Some(msg) => msg,
        None => get_last_message(&ctx).await?
    };

    owo_helper(ctx, &msg).await?;
    Ok(())
}

/// Context menu version of owo command.
// I decided to make this a context-menu command rather
// than having 2 commands that do the same thing
#[poise::command(context_menu_command = "uwu")]
pub async fn uwu(
    ctx:Context<'_>,
    msg: serenity::Message,
) -> Result<(), Error> {
    owo_helper(ctx, &msg).await?;
    Ok(())
}

/// Joe mama meme lololol
#[poise::command(prefix_command, slash_command)]
pub async fn whoisjoe(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("JOE MAMA").await?;
    Ok(())
}

/// Adds you to the list of immuwunized individuals (or removes you if you're already on it)
///
/// Like a vaccine for the UwU.
/// The difference here is that you can choose to deimmuwunize yourself.
#[poise::command(prefix_command, slash_command)]
pub async fn immuwune(ctx: Context<'_>) -> Result<(), Error> {
    let author = ctx.author().name.clone();

    let mut db = ctx.data().db.lock().await;
    
    let message = if db.immuwune.insert(author.clone()) {
        "You have successfully been immuwunized OwO"
    } else {
        db.immuwune.remove(&author);
        "You have successfully been deimmuwunized UwU"
    };
    
    db.save(&ctx.data().db_path).await?;

    ctx.reply(message).await?;
    Ok(())
}

/// Helper function for owo/uwu with immuwunity check
async fn owo_helper(
    ctx: Context<'_>,
    msg: &serenity::Message,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    
    // Check immuwunity for both message author and command author
    if db.immuwune.contains(&ctx.author().name) {
        ctx.say("UwU you'we immuwune, sowwy! 😭").await?;
    } else if db.immuwune.contains(&msg.author.name) {
        ctx.say("UwU dis usew is immuwune, sowwy! 😭").await?;
    } else {
        let transformed = owofy(&msg.content);
        ctx.say(&transformed).await?;
    }
    
    Ok(())
}

/// Subscribes to CatFacts!, Returns a random catfact.
/// Some of these are questionable
#[poise::command(prefix_command, slash_command)]
pub async fn catfact(ctx: Context<'_>) -> Result<(), Error> {
    let client = Client::new();
    let catfacts_url = String::from(
        "https://raw.githubusercontent.com/vadimdemedes/cat-facts/master/cat-facts.json",
    );

    let response: Value = client
        .get(catfacts_url)
        .header(USER_AGENT, "rust-web-api-client")
        .send()
        .await?
        .json()
        .await?;

    let message = response
        .as_array()
        .and_then(|a| {
            a.choose(&mut rand::rng())
                .and_then(|v| v.as_str())
        })
        .unwrap_or("I'm sorry, I couldn't fetch any catfacts!");

    ctx.say(message).await?;
    
    Ok(())
}
