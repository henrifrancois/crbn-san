use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[derive(Debug, poise::ChoiceParameter)]
pub enum TtsLanguage {
    #[name = "English"]
    English,
    #[name = "French"]
    French,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum TtsGender {
    #[name = "Male"]
    Male,
    #[name = "Female"]
    Female,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum TtsModel {
    #[name = "Google Standard"]
    GoogleStandard,
    #[name = "Piper"]
    Piper,
}

/// Configure Text-to-Speech settings
#[poise::command(
    slash_command,
    subcommands("voice", "gender", "model", "channel"),
    category = "Configuration"
)]
pub async fn tts(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the voice language and other voice-specific settings (Placeholder for now)
#[poise::command(slash_command)]
pub async fn voice(
    ctx: Context<'_>,
    #[description = "Language for TTS"] language: TtsLanguage,
    #[description = "Specific voice ID (optional)"] voice_id: Option<String>,
) -> Result<(), Error> {
    // TODO: Save settings to database
    ctx.say(format!("Set TTS language to {:?}. Voice ID: {:?}", language, voice_id)).await?;
    Ok(())
}

/// Set the speaker gender
#[poise::command(slash_command)]
pub async fn gender(
    ctx: Context<'_>,
    #[description = "Speaker gender"] gender: TtsGender,
) -> Result<(), Error> {
    // TODO: Save settings to database
    ctx.say(format!("Set TTS gender to {:?}", gender)).await?;
    Ok(())
}

/// Set the Text-to-Speech model
#[poise::command(slash_command)]
pub async fn model(
    ctx: Context<'_>,
    #[description = "TTS Model to use"] model: TtsModel,
) -> Result<(), Error> {
    // TODO: Save settings to database
    ctx.say(format!("Set TTS model to {:?}", model)).await?;
    Ok(())
}

/// Set the channel for Text-to-Speech usage (Admin only)
#[poise::command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn channel(
    ctx: Context<'_>,
    #[description = "The channel to use for TTS"] channel: serenity::Channel,
) -> Result<(), Error> {
    // TODO: Save settings to database
    ctx.say(format!("Set TTS channel to {}", channel)).await?;
    Ok(())
}
