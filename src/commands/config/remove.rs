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

  let guild_id = guild_id.to_string();
  let role_id = p_role.to_string();
  let count: Option<_> = sqlx::query!(
    "DELETE FROM roles WHERE guild_id = ? AND role_id = ? RETURNING *",
    guild_id,
    role_id
  )
  .fetch_optional(&state.pool)
  .await?;

  let formatted = if count.is_none() {
    format!(
      "Role <@&{}> is not a selfrole, so it cannot be removed.",
      p_role
    )
  } else {
    format!("Successfully removed selfrole <@&{}>", role_id)
  };

  let response = InteractionResponseDataBuilder::new()
    .flags(MessageFlags::EPHEMERAL)
    .content(formatted)
    .build();

  Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  })
}
