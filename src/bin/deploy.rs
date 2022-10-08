use json_gettext::{get_text, static_json_gettext_build, JSONGetText};
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

fn gdv(ctx: &JSONGetText, key: &str) -> String {
  get_text!(ctx, key).unwrap().to_string()
}

fn gtv<'a>(ctx: &'a JSONGetText<'a>, locale: &'a str, key: &'a str) -> (String, String) {
  (
    locale.to_string(),
    get_text!(ctx, locale, key).unwrap().to_string(),
  )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv()?;
  let ctx = static_json_gettext_build!(
    "en-US";
    "en-US" => "i18n/en-US.json",
    "pl" => "i18n/pl.json"
  )
  .unwrap();

  let roles_command = CommandBuilder::new(
    gdv(&ctx, "cmd::roles:name"),
    gdv(&ctx, "cmd::roles:desc"),
    CommandType::ChatInput,
  )
  .dm_permission(false)
  .name_localizations(vec![gtv(&ctx, "pl", "cmd::roles:name")])
  .description_localizations(vec![gtv(&ctx, "pl", "cmd::roles:desc")])
  .build();

  let persist_command = CommandBuilder::new(
    gdv(&ctx, "cmd::persist:name"),
    gdv(&ctx, "cmd::persist:desc"),
    CommandType::ChatInput,
  )
  .dm_permission(false)
  .default_member_permissions(Permissions::MANAGE_ROLES)
  .option(CommandOption::String(ChoiceCommandOptionData {
    name: gdv(&ctx, "cmd::persist::content:name"),
    description: gdv(&ctx, "cmd::persist::content:desc"),
    name_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::persist::content:name")]
        .into_iter()
        .collect()
    ),
    description_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::persist::content:desc")]
        .into_iter()
        .collect()
    ),
    ..Default::default()
  }))
  .name_localizations(vec![gtv(&ctx, "pl", "cmd::persist:name")])
  .description_localizations(vec![gtv(&ctx, "pl", "cmd::persist:desc")])
  .build();

  let config_command = CommandBuilder::new(
    gdv(&ctx, "cmd::config:name"),
    gdv(&ctx, "cmd::config:desc"),
    CommandType::ChatInput,
  )
  .dm_permission(false)
  .default_member_permissions(Permissions::MANAGE_ROLES)
  .option(CommandOption::SubCommand(OptionsCommandOptionData {
    name: gdv(&ctx, "cmd::config::add:name"),
    description: gdv(&ctx, "cmd::config::add:desc"),
    options: vec![
      CommandOption::Role(BaseCommandOptionData {
        name: gdv(&ctx, "cmd::config::add::role:name"),
        description: gdv(&ctx, "cmd::config::add::role:desc"),
        required: true,
        name_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::role:name")]
            .into_iter()
            .collect()
        ),
        description_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::role:desc")]
            .into_iter()
            .collect()
        )
      }),
      CommandOption::String(ChoiceCommandOptionData {
        name: gdv(&ctx, "cmd::config::add::label:name"),
        description: gdv(&ctx, "cmd::config::add::label:desc"),
        name_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::label:name")]
            .into_iter()
            .collect()
        ),
        description_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::label:desc")]
            .into_iter()
            .collect()
        ),
        ..Default::default()
      }),
      CommandOption::String(ChoiceCommandOptionData {
        name: gdv(&ctx, "cmd::config::add::description:name"),
        description: gdv(&ctx, "cmd::config::add::description:desc"),
        name_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::description:name")]
            .into_iter()
            .collect()
        ),
        description_localizations: Some(
          vec![gtv(&ctx, "pl", "cmd::config::add::description:desc")]
            .into_iter()
            .collect()
        ),
        ..Default::default()
      }),
    ],
    name_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::config::add:name")]
        .into_iter()
        .collect()
    ),
    description_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::config::add:desc")]
        .into_iter()
        .collect()
    ),
  }))
  .option(CommandOption::SubCommand(OptionsCommandOptionData {
    name: gdv(&ctx, "cmd::config::remove:name"),
    description: gdv(&ctx, "cmd::config::remove:desc"),
    options: vec![CommandOption::Role(BaseCommandOptionData {
      name: gdv(&ctx, "cmd::config::remove::role:name"),
      description: gdv(&ctx, "cmd::config::remove::role:desc"),
      required: true,
      name_localizations: Some(
        vec![gtv(&ctx, "pl", "cmd::config::remove::role:name")]
          .into_iter()
          .collect()
      ),
      description_localizations: Some(
        vec![gtv(&ctx, "pl", "cmd::config::remove::role:desc")]
          .into_iter()
          .collect()
      ),
    })],
    name_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::config::remove:name")]
        .into_iter()
        .collect()
    ),
    description_localizations: Some(
      vec![gtv(&ctx, "pl", "cmd::config::remove:desc")]
        .into_iter()
        .collect()
    )
  }))
  .name_localizations(vec![gtv(&ctx, "pl", "cmd::config:name")])
  .description_localizations(vec![gtv(&ctx, "pl", "cmd::config:desc")])
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
