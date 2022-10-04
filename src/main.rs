use futures::StreamExt;
use sqlx::SqlitePool;
use std::{collections::HashSet, env, sync::Arc};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_http::{client::ClientBuilder, request::AuditLogReason, Client};
use twilight_model::{
  application::{
    command::CommandType,
    component::{select_menu::SelectMenuOption, ActionRow, Component, ComponentType, SelectMenu},
    interaction::{
      application_command::{CommandData, CommandDataOption, CommandOptionValue},
      message_component::MessageComponentInteractionData,
      InteractionData,
    },
  },
  channel::message::{AllowedMentions, MessageFlags},
  gateway::payload::incoming::InteractionCreate,
  http::interaction::{InteractionResponse, InteractionResponseType},
  id::{
    marker::{ApplicationMarker, GuildMarker},
    Id,
  },
};
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Debug, Clone)]
struct State {
  pool: SqlitePool,
  client: Arc<Client>,
  app_id: Id<ApplicationMarker>,
}

struct RoleData {
  role_id: String,
  label: String,
  description: Option<String>,
}

#[tracing::instrument(ret, level = "debug", skip_all)]
async fn roles_command(
  state: State,
  guild_id: Id<GuildMarker>
) -> anyhow::Result<InteractionResponse> {
  let guild_id = guild_id.to_string();
  let self_roles: Vec<RoleData> = sqlx::query_as!(
    RoleData,
    "SELECT role_id, label, description FROM roles WHERE guild_id = ?",
    guild_id,
  )
  .fetch_all(&state.pool)
  .await?;

  if self_roles.is_empty() {
    let response = InteractionResponseDataBuilder::new()
      .flags(MessageFlags::EPHEMERAL)
      .content("Sorry, there are no roles to pick from. Contact server administrator to see if this is intentional.")
      .build();

    return Ok(InteractionResponse {
      data: Some(response),
      kind: InteractionResponseType::ChannelMessageWithSource,
    });
  }

  let select = Component::ActionRow(ActionRow {
    components: vec![Component::SelectMenu(SelectMenu {
      custom_id: "roleMenu".to_string(),
      disabled: false,
      max_values: Some(self_roles.len().try_into().unwrap()),
      min_values: Some(0),
      placeholder: Some("Select your roles".to_string()),
      options: self_roles
        .into_iter()
        .map(|role| SelectMenuOption {
          default: false,
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

#[tracing::instrument(ret, skip_all)]
async fn config_add_command(
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
  let p_label = options
    .iter()
    .find_map(|option| match &option.value {
      CommandOptionValue::String(label) if option.name == "label" => Some(label),
      _ => None,
    })
    .unwrap();
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
  sqlx::query!(
    "INSERT INTO roles VALUES (?, ?, ?, ?);",
    guild_id,
    role_id,
    p_label,
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

#[tracing::instrument(ret, skip_all)]
async fn config_rm_command(
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

#[tracing::instrument(ret, skip_all)]
async fn handle_command(
  state: State,
  command: &Box<CommandData>,
  interaction: impl AsRef<InteractionCreate>
) -> anyhow::Result<InteractionResponse> {
  let interaction = interaction.as_ref();
  match (command.kind, command.name.as_str()) {
    (CommandType::ChatInput, "roles") => roles_command(state, interaction.guild_id.unwrap()).await,
    (CommandType::ChatInput, "config") => match command.options.get(0) {
      Some(CommandDataOption {
        name,
        value: CommandOptionValue::SubCommand(options),
      }) if name == "add" => config_add_command(state, options, interaction.guild_id.unwrap()).await,
      Some(CommandDataOption {
        name,
        value: CommandOptionValue::SubCommand(options),
      }) if name == "remove" => config_rm_command(state, options, interaction.guild_id.unwrap()).await,
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
}

#[tracing::instrument(ret, level = "debug", skip_all)]
async fn roles_select(
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

#[tracing::instrument(ret, skip_all)]
async fn handle_menu(
  state: State,
  interaction: Box<InteractionCreate>,
  component: &MessageComponentInteractionData,
) -> anyhow::Result<InteractionResponse> {
  match (component.component_type, component.custom_id.as_str()) {
    (ComponentType::SelectMenu, "roleMenu") => roles_select(state, interaction, component).await,
    _ => unreachable!(),
  }
}

#[tracing::instrument(ret, skip_all)]
async fn interaction_dispatcher(
  state: State,
  interaction: Box<InteractionCreate>,
) -> anyhow::Result<()> {
  let response = match interaction.data {
    Some(InteractionData::ApplicationCommand(ref command)) => {
      handle_command(state.clone(), command, &interaction).await?
    }
    Some(InteractionData::MessageComponent(ref component)) => {
      handle_menu(state.clone(), interaction.clone(), component).await?
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv()?;
  tracing_subscriber::fmt::init();

  let (cluster, mut events) = Cluster::new(env::var("TOKEN")?, Intents::empty()).await?;
  let cluster = Arc::new(cluster);

  let cluster_spawn = Arc::clone(&cluster);

  tokio::spawn(async move {
    cluster_spawn.up().await;
  });

  let client: Arc<_> = ClientBuilder::new()
    .default_allowed_mentions(AllowedMentions::default())
    .token(env::var("TOKEN")?)
    .build()
    .into();

  let state = State {
    pool: SqlitePool::connect("sqlite:db.db").await?,
    app_id: client
      .current_user_application()
      .exec()
      .await?
      .model()
      .await?
      .id,
    client,
  };

  while let Some((id, event)) = events.next().await {
    // println!("Shard: {id}, Event: {:?}", event.kind());
    match event {
      Event::InteractionCreate(interaction) => {
        let state = state.clone();
        tokio::spawn(async move {
          if let Err(error) = interaction_dispatcher(state, interaction).await {
            tracing::error!("{}", error);
          }
        });
      }
      _ => tracing::debug!("Shard: {id}, Event: {:?}", event.kind()),
    }
  }

  Ok(())
}
