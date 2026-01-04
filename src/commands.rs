use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::{Context, Error};
use poise::serenity_prelude::ReactionType;
use poise::{serenity_prelude::{self as serenity, EditRole, Role, RoleId, Mentionable}, CreateReply};
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

/// Ping the bot
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
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

    // Need to get prefix
    // Then stick it onto the beginning of whatever suggestions come back
    let (prefix, current) = match partial.rfind(' ') {
        Some(i) => (&partial[..=i], &partial[i + 1..]),
        None => ("", partial),
    };

    guild_roles
        .into_values()
        .map(move |r| format!("{prefix}{}", r.name))
        .filter(move |s| s.contains(current))
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

/// Delete role(s)
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn del(
    ctx: Context<'_>,
    #[rest]
    #[description = "Role(s) to delete"] 
    #[autocomplete = "autocomplete_role"]
    roles: String,
) -> Result<(), Error> {
    let member = ctx.author_member().await.ok_or("Unable to get member")?;
    let member_roles = member.roles(ctx).ok_or("Member has no roles")?;

    let mut deleted_roles = String::new();
    let mut unsuccessful_roles = String::new();
    for role in roles.split_whitespace() {
        if let Some(role_id) = member_roles.iter().find(|r| r.name == role) {
            member.remove_role(&ctx, role_id).await?;
            deleted_roles.push_str(role);
            deleted_roles.push(' ');
        } else {
            unsuccessful_roles.push_str(role);
            unsuccessful_roles.push(' ');
        }
    }

    if !unsuccessful_roles.is_empty() {
        ctx.say(format!("**Unable** to delete: {}", unsuccessful_roles)).await?;
    }
    if !deleted_roles.is_empty() {
        ctx.say(format!("Successfully deleted: {}", deleted_roles)).await?;
    }

    Ok(())
}

/// Helper function to check if a vector has duplicates.
// Found this online
fn has_duplicates<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    !iter.into_iter().all(|x| uniq.insert(x))
}

/// Helper function to create multiple roles. Hidden from help menu 
/// because only for privileged users.
#[poise::command(prefix_command, slash_command, guild_only, hide_in_help,
required_permissions = "MANAGE_ROLES")]
pub async fn create_roles(
    ctx: Context<'_>,
    #[rest]
    #[description = "Names of role(s) you wish to create"]
    roles: String
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Unable to get guild_id")?;
    let existing_roles = guild_id.roles(&ctx).await?;

    // Do not allow roles with duplicate names to be created
    if has_duplicates(roles.split_whitespace())
        || existing_roles.values().any(|r| roles.contains(&r.name)) {
        ctx.say("Duplicate role(s) found, unable to create").await?;
        return Ok(());
    }
    
    let mut created_roles = String::new();
    for role in roles.split_whitespace() {
        let builder = EditRole::new().name(role).mentionable(true);
        guild_id.create_role(&ctx, builder).await?;
        created_roles.push_str(role);
        created_roles.push(' ');
    }

    ctx.say(format!("Successfully created roles: {created_roles}")).await?;

    Ok(())
}

/// List roles of user that called this command
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn my_roles(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let member = ctx.author_member().await.ok_or("Unable to get guild member")?;
    let message = if let Some(roles) = member.roles(ctx) {
        let roles_str = roles
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>()
            .join("\n- ");
        format!("Your roles:\n- {roles_str}")
    } else {
        String::from("You haven't added any roles yet!")
    };

    ctx.say(message).await?;

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
    arg2: Option<String>,
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
    let dvalue: u32 = if let Some(dtype) = arg2.clone() && dtype.starts_with('d') {
        dtype.split_once('d')
            .map(|pair| pair.1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(6)
    } else {
        arg2
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
