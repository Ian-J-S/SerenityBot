use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::{Context, Error};
use poise::{serenity_prelude::{EditRole, Role, RoleId}};

/// List server roles
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

/// Autocomplete function when typing roles for add / del
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
