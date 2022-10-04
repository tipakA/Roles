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
    let formatted = format!(
      "You cannot add a managed role <@&{}> to selfroles.",
      found.id
    );

    let response = InteractionResponseDataBuilder::new()
      .flags(MessageFlags::EPHEMERAL)
      .content(formatted)
      .build();

    return Ok(InteractionResponse {
      data: Some(response),
      kind: InteractionResponseType::ChannelMessageWithSource,
    });
  }

  if found.id.cast() == guild_id {
    let response = InteractionResponseDataBuilder::new()
      .flags(MessageFlags::EPHEMERAL)
      .content("You cannot add @everyone role to selfroles.")
      .build();

    return Ok(InteractionResponse {
      data: Some(response),
      kind: InteractionResponseType::ChannelMessageWithSource,
    });
  }

  let my_roles = guild_roles
    .iter()
    .filter(|role| me.roles.contains(&role.id));

  let my_highest = my_roles.max().unwrap();

  if my_highest <= found {
    let formatted = format!(
      "You cannot add role <@&{}> to selfroles as it is higher than my highest role <@&{}>.",
      found.id, my_highest.id
    );

    let response = InteractionResponseDataBuilder::new()
      .flags(MessageFlags::EPHEMERAL)
      .content(formatted)
      .build();

    return Ok(InteractionResponse {
      data: Some(response),
      kind: InteractionResponseType::ChannelMessageWithSource,
    });
  }

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

  let formatted = format!("Successfully added selfrole <@&{}>.", found.id);
  let response = InteractionResponseDataBuilder::new()
    .content(formatted)
    .build();

  return Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  });
}
