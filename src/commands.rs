use twilight_model::{
  application::{
    command::CommandType,
    component::ComponentType,
    interaction::{
      application_command::{CommandData, CommandDataOption, CommandOptionValue},
      message_component::MessageComponentInteractionData,
    },
  },
  gateway::payload::incoming::InteractionCreate,
  http::interaction::InteractionResponse,
};

use crate::State;

pub mod config;
pub mod roles;

#[tracing::instrument(ret, skip_all)]
pub async fn handle_command(
  state: State,
  command: &Box<CommandData>,
  interaction: impl AsRef<InteractionCreate>,
) -> anyhow::Result<InteractionResponse> {
  let interaction = interaction.as_ref();
  match (command.kind, command.name.as_str()) {
    (CommandType::ChatInput, "roles") => {
      roles::roles_command(
        state,
        interaction.guild_id.unwrap(),
        interaction.author_id().unwrap(),
      )
      .await
    }
    (CommandType::ChatInput, "persist") => roles::persist(),
    (CommandType::ChatInput, "config") => match command.options.get(0) {
      Some(CommandDataOption {
        name,
        value: CommandOptionValue::SubCommand(options),
      }) if name == "add" => {
        config::add::config_add_command(state, options, interaction.guild_id.unwrap()).await
      }
      Some(CommandDataOption {
        name,
        value: CommandOptionValue::SubCommand(options),
      }) if name == "remove" => {
        config::remove::config_rm_command(state, options, interaction.guild_id.unwrap()).await
      }
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
}

#[tracing::instrument(ret, skip_all)]
pub async fn handle_menu(
  state: State,
  interaction: Box<InteractionCreate>,
  component: &MessageComponentInteractionData,
) -> anyhow::Result<InteractionResponse> {
  match (component.component_type, component.custom_id.as_str()) {
    (ComponentType::SelectMenu, "roleMenu") => {
      roles::select::roles_select(state, interaction, component).await
    }
    (ComponentType::Button, "selectRoles") => {
      roles::roles_command(
        state,
        interaction.guild_id.unwrap(),
        interaction.author_id().unwrap(),
      )
      .await
    }
    _ => unreachable!(),
  }
}
