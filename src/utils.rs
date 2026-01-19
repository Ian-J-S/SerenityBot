use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, GetMessages};

/// Helper function to get the author of the last message in the current channel
pub async fn get_last_message(ctx: &Context<'_>) -> Result<serenity::Message, Error> {
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
