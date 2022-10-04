use twilight_model::{
  application::interaction::InteractionData, gateway::payload::incoming::InteractionCreate,
};

use crate::{commands, State};

#[tracing::instrument(ret, skip_all)]
pub async fn interaction_dispatcher(
  state: State,
  interaction: Box<InteractionCreate>,
) -> anyhow::Result<()> {
  let response = match interaction.data {
    Some(InteractionData::ApplicationCommand(ref command)) => {
      commands::handle_command(state.clone(), command, &interaction).await?
    }
    Some(InteractionData::MessageComponent(ref component)) => {
      commands::handle_menu(state.clone(), interaction.clone(), component).await?
    }
    _ => unreachable!(),
  };

  let client = state.client.interaction(state.app_id);
  client
    .create_response(interaction.id, &interaction.token, &response)
    .exec()
    .await?
    .text()
    .await?;

  Ok(())
}
