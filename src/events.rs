use twilight_http::{api_error::ApiError, error::ErrorType};
use twilight_model::{
  application::interaction::InteractionData,
  channel::message::MessageFlags,
  gateway::payload::incoming::InteractionCreate,
  http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{commands, State};

#[tracing::instrument(ret, skip_all)]
pub async fn interaction_dispatcher(
  state: State,
  interaction: Box<InteractionCreate>,
) -> anyhow::Result<()> {
  let response = match interaction.data {
    Some(InteractionData::ApplicationCommand(ref command)) => {
      commands::handle_command(state.clone(), command, &interaction).await
    }
    Some(InteractionData::MessageComponent(ref component)) => {
      commands::handle_menu(state.clone(), interaction.clone(), component).await
    }
    _ => unreachable!(),
  }
  .unwrap_or_else(|err| {
    let err_message = match err.downcast::<twilight_http::Error>() {
      Ok(e) => match e.kind() {
        ErrorType::Response {
          error: ApiError::General(error),
          ..
        } => error.message.clone(),
        _ => e.to_string(),
      },
      Err(e) => e.to_string(),
    };

    InteractionResponse {
      kind: InteractionResponseType::ChannelMessageWithSource,
      data: Some(
        InteractionResponseDataBuilder::new()
          .content(format!("Sorry, an error occured: {}\nPlease contact an administrator about this.", err_message))
          .flags(MessageFlags::EPHEMERAL)
          .build(),
      ),
    }
  });

  let client = state.client.interaction(state.app_id);
  client
    .create_response(interaction.id, &interaction.token, &response)
    .exec()
    .await?
    .text()
    .await?;

  Ok(())
}
