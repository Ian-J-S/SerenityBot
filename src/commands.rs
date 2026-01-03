use std::collections::HashMap;

use crate::{Context, Error};
use poise::{serenity_prelude::{self as serenity, RoleId, Role}, CreateReply};
use rand::Rng;

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

/// Shut down the bot. Owners only.
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

/// Crab party
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

/// Echo content of a message
#[poise::command(prefix_command, context_menu_command = "Echo", slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    ctx.say(&msg.content).await?;
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

/// List roles
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn list_roles(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Unable to get guild ID")?;
    let roles = guild_id.roles(&ctx).await?;

    // Convert roles hashmap into vector of strings, ignore some roles
    let mut roles: Vec<String> = roles.iter()
        .filter_map(|r| {
            let role_name = r.1.name.clone();
            if role_name == "@everyone" {
                None
            } else {
                Some(role_name.clone())
            }
        })
        .collect();

    // Sort and convert to newline delimited string
    roles.sort();
    let roles_str = format!("Server roles:\n- {}", roles.join("\n- "));

    ctx.say(roles_str).await?;

    Ok(())
}

/// Get a RoleID based on a role name string.
// There is probably a better way to do this.
fn get_role_id(target_role: &str, roles: &HashMap<RoleId, Role>) -> Option<RoleId> {
    for role in roles {
        if role.1.name == target_role {
            return Some(*role.0)
        }
    }
    None
}

async fn autocomplete_role<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = String> + 'a {
    let guild_id = ctx.guild_id().expect("Unable to get guild ID");
    let guild_roles = guild_id.roles(&ctx).await
        .expect("Unable to get guild roles");

    guild_roles
        .into_values()
        .map(|r| r.name)
        .filter(move |s| s.starts_with(partial))
}

/// Add role(s)
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[rest]
    #[description = "Role(s) to add"] 
    #[autocomplete = "autocomplete_role"]
    roles: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Unable to get guild ID")?;
    let guild_roles = guild_id.roles(&ctx).await?;

    let mut added_roles = String::new();
    let mut unsuccessful_roles = String::new();
    for role in roles.split_whitespace() {
        // If role does not exist
        if !guild_roles.values().any(|r| r.name == role) {
            unsuccessful_roles.push_str(role);
            unsuccessful_roles.push(' ');
        } else {
            let user = ctx.author_member().await.ok_or("Unable to get Member")?;
            if let Some(id) = get_role_id(role, &guild_roles) {
                user.add_role(&ctx, id).await?;
                added_roles.push_str(role);
                added_roles.push(' ');
            // Failed to get ID
            } else {
                unsuccessful_roles.push_str(role);
                unsuccessful_roles.push(' ');
            }
        }
    }

    if !unsuccessful_roles.is_empty() {
        ctx.say(format!("**Unable** to add: {}", unsuccessful_roles)).await?;
    }
    if !added_roles.is_empty() {
        ctx.say(format!("Successfully added: {}", added_roles)).await?;
    }

    Ok(())
}
