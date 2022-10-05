use twilight_model::{
  application::interaction::application_command::{CommandDataOption, CommandOptionValue},
  channel::message::MessageFlags,
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
  let mut errored = false;
  let mut formatted = "".to_string();

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

  if found.managed {
    errored = true;
    formatted = format!(
      "You cannot add a managed role <@&{}> to selfroles.",
      found.id
    );
  }

  if found.id.cast() == guild_id {
    errored = true;
    formatted = "You cannot add @everyone role to selfroles.".to_string();
  }

  let my_roles = guild_roles
    .iter()
    .filter(|role| me.roles.contains(&role.id));

  let my_highest = my_roles.max().unwrap();

  if my_highest <= found {
    errored = true;
    formatted = format!(
      "You cannot add role <@&{}> to selfroles as it is higher than my highest role <@&{}>.",
      found.id, my_highest.id
    );
  }

  let mut response = InteractionResponseDataBuilder::new();

  if errored {
    response = response.content(formatted).flags(MessageFlags::EPHEMERAL);
  } else {
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

    formatted = format!("Successfully added selfrole <@&{}>.", found.id);
    response = response.content(formatted);
  }

  return Ok(InteractionResponse {
    data: Some(response.build()),
    kind: InteractionResponseType::ChannelMessageWithSource,
  });
}
