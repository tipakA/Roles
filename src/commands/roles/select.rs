use std::collections::HashSet;
use twilight_http::request::AuditLogReason;
use twilight_model::{
  application::interaction::message_component::MessageComponentInteractionData,
  channel::message::MessageFlags,
  gateway::payload::incoming::InteractionCreate,
  http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{RoleData, State};

#[tracing::instrument(ret, level = "debug", skip_all)]
pub async fn roles_select(
  state: State,
  interaction: Box<InteractionCreate>,
  component: &MessageComponentInteractionData,
) -> anyhow::Result<InteractionResponse> {
  let guild_id = interaction.guild_id.unwrap().to_string();
  let self_roles: Vec<RoleData> = sqlx::query_as!(
    RoleData,
    "SELECT role_id, label, description FROM roles WHERE guild_id = ?",
    guild_id
  )
  .fetch_all(&state.pool)
  .await?;

  let current_roles = interaction
    .member
    .as_ref()
    .unwrap()
    .roles
    .iter()
    .cloned()
    .collect::<HashSet<_>>();
  let self_roles = self_roles
    .into_iter()
    .map(|role| role.role_id.parse())
    .collect::<Result<HashSet<_>, _>>()?;

  let other_roles = current_roles.difference(&self_roles).cloned().map(Ok);
  let final_roles = other_roles
    .chain(component.values.iter().map(|role| role.parse()))
    .collect::<Result<Vec<_>, _>>()?;

  state
    .client
    .update_guild_member(
      interaction.guild_id.unwrap(),
      interaction.author_id().unwrap(),
    )
    .roles(&final_roles)
    .reason("self role")?
    .exec()
    .await?;

  let mapped = component
    .values
    .iter()
    .map(|role| format!("<@&{}>", role))
    .collect::<Vec<_>>();
  let out = if mapped.is_empty() {
    String::from("Cleared your roles.")
  } else {
    format!("Set your roles to {}.", mapped.join(", "))
  };

  let response = InteractionResponseDataBuilder::new()
    .flags(MessageFlags::EPHEMERAL)
    .content(out)
    .build();

  Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  })
}
