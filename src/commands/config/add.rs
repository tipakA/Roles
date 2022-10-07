use twilight_model::{
  application::interaction::application_command::{CommandDataOption, CommandOptionValue},
  http::interaction::{InteractionResponse, InteractionResponseType},
  id::{marker::GuildMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::State;

#[tracing::instrument(ret, skip_all)]
pub async fn exec(
  state: State,
  options: &[CommandDataOption],
  guild_id: Id<GuildMarker>,
) -> anyhow::Result<InteractionResponse> {
  let p_role = options
    .iter()
    .find_map(|option| match option.value {
      CommandOptionValue::Role(role) if option.name == "role" => Some(role),
      _ => None,
    })
    .unwrap();
  let p_label = options.iter().find_map(|option| match &option.value {
    CommandOptionValue::String(label) if option.name == "label" => Some(label),
    _ => None,
  });
  let p_description = options.iter().find_map(|option| match &option.value {
    CommandOptionValue::String(label) if option.name == "description" => Some(label),
    _ => None,
  });

  let me = state
    .client
    .guild_member(guild_id, state.app_id.cast())
    .exec()
    .await?
    .model()
    .await?;

  let guild_roles = state.client.roles(guild_id).exec().await?.model().await?;

  let found = guild_roles
    .iter()
    .find(|role| role.id == p_role)
    .ok_or_else(|| anyhow::anyhow!("Couldn't find the selected role."))?;

  anyhow::ensure!(
    !found.managed,
    "You cannot add a managed role <@&{}> to selfroles.",
    found.id
  );

  anyhow::ensure!(
    found.id.cast() != guild_id,
    "You cannot add @everyone role to selfroles."
  );

  let my_highest = guild_roles
    .iter()
    .filter(|role| me.roles.contains(&role.id))
    .max();

  anyhow::ensure!(
    my_highest < Some(found),
    "You cannot add role <@&{}> to selfroles as it is higher than, or equally high as my highest role <@&{}>.",
    found.id,
    my_highest.map(|r| r.id).unwrap_or(guild_id.cast())
  );

  let guild_id = guild_id.to_string();
  let role_id = found.id.to_string();
  let role_name = p_label.unwrap_or(&found.name);
  sqlx::query!(
    r#"
      INSERT INTO roles VALUES (?, ?, ?, ?)
      ON CONFLICT (role_id) DO UPDATE SET
        label = excluded.label,
        description = excluded.description
      ;
    "#,
    guild_id,
    role_id,
    role_name,
    p_description
  )
  .execute(&state.pool)
  .await?;

  let response = InteractionResponseDataBuilder::new()
    .content(format!("Successfully added selfrole <@&{}>.", found.id))
    .build();

  return Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  });
}
