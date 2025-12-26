use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

pub fn register() -> CreateCommand {
    CreateCommand::new("welcome")
        .description("Welcome a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "The user to welcome.")
                .required(true)
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "The welcoming message")
                .required(true)
                .add_string_choice("standard", "Welcome to Caricabana!")
                .add_string_choice("simple", "YO")
        )

}