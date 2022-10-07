use std::fmt::Display;
use twilight_http::{api_error::ApiError, error::ErrorType};
use twilight_model::{
  application::interaction::InteractionData,
  channel::message::MessageFlags,
  gateway::payload::incoming::InteractionCreate,
  http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{commands, State};

fn format_error(error: impl Display) -> String {
  format!(
    "Sorry, an unexpected error occured: {}\nPlease contact an administrator about this.",
    error
  )
}

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
    let err_message = if let Some(e) = err.downcast_ref::<twilight_http::Error>() {
      match e.kind() {
        ErrorType::Response {
          error: ApiError::General(error),
          ..
        } => format_error(&error.message),
        _ => format_error(e),
      }
    } else if let Some(e) = err.downcast_ref::<sqlx::Error>() {
      format_error(e)
    } else {
      err.to_string()
    };

    InteractionResponse {
      kind: InteractionResponseType::ChannelMessageWithSource,
      data: Some(
        InteractionResponseDataBuilder::new()
          .content(err_message)
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
