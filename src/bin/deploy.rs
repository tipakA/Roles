use std::env;
use twilight_http::client::ClientBuilder;
use twilight_model::{
  application::command::{
    BaseCommandOptionData, ChoiceCommandOptionData, CommandOption, CommandType,
    OptionsCommandOptionData,
  },
  guild::Permissions,
};
use twilight_util::builder::command::CommandBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv()?;
  let roles_command = CommandBuilder::new("roles", "Select roles you want", CommandType::ChatInput)
    .dm_permission(false)
    .build();

  let persist_command = CommandBuilder::new(
    "persist",
    "Make a button to get roles",
    CommandType::ChatInput,
  )
  .dm_permission(false)
  .default_member_permissions(Permissions::MANAGE_ROLES)
  .build();

  let config_command = CommandBuilder::new(
    "config",
    "Manage selfroles for the server",
    CommandType::ChatInput,
  )
  .dm_permission(false)
  .default_member_permissions(Permissions::MANAGE_ROLES)
  .option(CommandOption::SubCommand(OptionsCommandOptionData {
    name: "add".to_string(),
    description: "Add new selfrole, or update existing one".to_string(),
    options: vec![
      CommandOption::Role(BaseCommandOptionData {
        name: "role".to_string(),
        description: "Select a role".to_string(),
        required: true,
        ..Default::default()
      }),
      CommandOption::String(ChoiceCommandOptionData {
        name: "label".to_string(),
        description: "Role name that will be displayed in the select menu".to_string(),
        ..Default::default()
      }),
      CommandOption::String(ChoiceCommandOptionData {
        name: "description".to_string(),
        description: "Optional description displayed in the select menu".to_string(),
        ..Default::default()
      }),
    ],
    ..Default::default()
  }))
  .option(CommandOption::SubCommand(OptionsCommandOptionData {
    name: "remove".to_string(),
    description: "Remove a selfrole".to_string(),
    options: vec![CommandOption::Role(BaseCommandOptionData {
      name: "role".to_string(),
      description: "Select a role".to_string(),
      required: true,
      ..Default::default()
    })],
    ..Default::default()
  }))
  .build();

  let client = ClientBuilder::new().token(env::var("TOKEN")?).build();

  let app_id = client
    .current_user_application()
    .exec()
    .await?
    .model()
    .await?
    .id;

  client
    .interaction(app_id)
    .set_global_commands(&[roles_command, persist_command, config_command])
    .exec()
    .await?;

  Ok(())
}
