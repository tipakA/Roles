use futures::StreamExt;
use sqlx::SqlitePool;
use std::{env, sync::Arc};
use twilight_gateway::{Cluster, Event, Intents};
use twilight_http::{client::ClientBuilder, Client};
use twilight_model::{
  channel::message::AllowedMentions,
  id::{marker::ApplicationMarker, Id},
};

pub mod commands;
pub mod events;
pub mod util;

#[derive(Debug, Clone)]
pub struct State {
  pool: SqlitePool,
  client: Arc<Client>,
  app_id: Id<ApplicationMarker>,
}

pub struct RoleData {
  role_id: String,
  label: String,
  description: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  if let Err(error) = dotenvy::dotenv() {
    tracing::warn!("Missing .env file\n{}", error);
  }

  tracing_subscriber::fmt::init();

  let (cluster, mut events) = Cluster::new(env::var("TOKEN")?, Intents::GUILDS).await?;
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
          if let Err(error) = events::interaction_dispatcher(state, interaction).await {
            tracing::error!("{}", error);
          }
        });
      }
      _ => tracing::debug!("Shard: {id}, Event: {:?}", event.kind()),
    }
  }

  Ok(())
}
