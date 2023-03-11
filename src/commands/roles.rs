use twilight_model::{
  application::{
    component::{
      button::ButtonStyle, select_menu::SelectMenuOption, ActionRow, Button, Component, SelectMenu,
    },
    interaction::application_command::{CommandData, CommandOptionValue},
  },
  channel::message::MessageFlags,
  http::interaction::{InteractionResponse, InteractionResponseType},
  id::{
    marker::{GuildMarker, UserMarker},
    Id,
  },
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{RoleData, State};

pub mod select;

#[tracing::instrument(ret, level = "debug", skip_all)]
pub async fn exec(
  state: State,
  guild_id: Id<GuildMarker>,
  user_id: Id<UserMarker>,
) -> anyhow::Result<InteractionResponse> {
  let guild_id_string = guild_id.to_string();
  let self_roles: Vec<RoleData> = sqlx::query_as!(
    RoleData,
    "SELECT role_id, label, description FROM roles WHERE guild_id = ?",
    guild_id_string,
  )
  .fetch_all(&state.pool)
  .await?;

  anyhow::ensure!(!self_roles.is_empty(), "Sorry, there are no roles to pick from. Contact server administrator to check if this is intentional.");
  // anyhow::ensure!(!self_roles.is_empty(), localize(locale, "noRoles"));

  let guild_roles = state.client.roles(guild_id).exec().await?.model().await?;
  let mut sorted_self_roles = self_roles;
  sorted_self_roles.sort_unstable_by(|a, b| {
    let a_role_idx = guild_roles.iter().position(|x| x.id.to_string() == a.role_id).unwrap();
    let b_role_idx = guild_roles.iter().position(|x| x.id.to_string() == b.role_id).unwrap();

    let a_role = guild_roles.get(a_role_idx).unwrap();
    let b_role = guild_roles.get(b_role_idx).unwrap();

    b_role.partial_cmp(a_role).unwrap_or(std::cmp::Ordering::Equal)
  });

  let member = state
    .client
    .guild_member(guild_id, user_id)
    .exec()
    .await?
    .model()
    .await?;

  let select = Component::ActionRow(ActionRow {
    components: vec![Component::SelectMenu(SelectMenu {
      custom_id: "roleMenu".to_string(),
      disabled: false,
      max_values: Some(sorted_self_roles.len().try_into().unwrap()),
      min_values: Some(0),
      placeholder: Some("Select your roles".to_string()),
      options: sorted_self_roles
        .into_iter()
        .map(|role| SelectMenuOption {
          default: member.roles.contains(&role.role_id.parse().unwrap()),
          description: role.description,
          emoji: None,
          label: role.label,
          value: role.role_id,
        })
        .collect(),
    })],
  });

  let response = InteractionResponseDataBuilder::new()
    .components([select])
    .flags(MessageFlags::EPHEMERAL)
    .content("Select all the roles you want, and click out of the menu to confirm.")
    .build();

  Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  })
}

pub fn persist(command: &Box<CommandData>) -> anyhow::Result<InteractionResponse> {
  let p_content = command
    .options
    .iter()
    .find_map(|option| match &option.value {
      CommandOptionValue::String(content) if option.name == "content" => Some(content),
      _ => None,
    });

  let button = Component::ActionRow(ActionRow {
    components: vec![Component::Button(Button {
      custom_id: Some("selectRoles".to_string()),
      disabled: false,
      emoji: None,
      label: Some("Get Roles".to_string()),
      style: ButtonStyle::Primary,
      url: None,
    })],
  });

  let response = InteractionResponseDataBuilder::new()
    .components([button])
    .content(p_content.unwrap_or(&"GET ROLES HERE".to_string()))
    .build();

  Ok(InteractionResponse {
    data: Some(response),
    kind: InteractionResponseType::ChannelMessageWithSource,
  })
}
