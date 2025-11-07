use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    api::{
        app::{AppService, AppState},
        error::AppError,
    },
    core::{
        agents::AgentService,
        auth::{AuthConfig, AuthService},
        client::{run_client_server, ClientService},
        core::Core,
        matches::MatchService,
    },
    repo::{
        agents::AgentRepo, game_type::GameTypeRepo, matches::MatchRepo,
        participation::ParticipationRepo, turns::TurnRepo, users::UserRepo,
    },
};

pub mod api;
pub mod core;
pub mod repo;

use rust_embed::RustEmbed;

// 🚨 请根据您的实际路径修改 #[folder]
// 假设您的 Cargo.toml 在 my_rust_project/，而 dist 在 my_rust_project/frontend/dist/
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct Asset;

pub async fn setup_database() -> PgPool {
    let database_url = "postgres://tacklebox:password@localhost/mydb";

    PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to create PostgreSQL connection pool")
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let auth_config = AuthConfig {
        jwt_secret: "hello world".to_string(),
        expiration: 3600,
    };

    let pool = Arc::new(setup_database().await);
    let gametype_repo = Arc::new(GameTypeRepo { pool: pool.clone() });
    let agent_repo = Arc::new(AgentRepo { pool: pool.clone() });
    let user_repo = Arc::new(UserRepo { pool: pool.clone() });
    let match_repo = Arc::new(MatchRepo { pool: pool.clone() });
    let turn_repo = Arc::new(TurnRepo { pool: pool.clone() });
    let participation_repo = Arc::new(ParticipationRepo { pool: pool.clone() });

    let auth_service = AuthService {
        user_repo: user_repo.clone(),
        config: auth_config,
    };

    let agent_service = AgentService {
        repo: Arc::new(AgentRepo { pool: pool.clone() }),
    };
    let sponsor_urls = HashMap::from_iter(vec![(
        "rlcard".to_string(),
        "http://localhost:50051".to_string(),
    )]);

    let mut core = Core::new(
        sponsor_urls,
        match_repo.clone(),
        agent_repo.clone(),
        turn_repo.clone(),
    )
    .await?;
    let core_tx = core.tx();
    tokio::spawn(async move {
        let _ = core.run().await;
    });
    let client_service = ClientService::new(core_tx.clone()).await?;
    let match_service = MatchService::new(
        gametype_repo,
        user_repo,
        agent_repo,
        match_repo,
        turn_repo,
        participation_repo,
        core_tx,
    );

    let app_state = AppState {
        agent_service: Arc::new(agent_service),
        auth_service: Arc::new(auth_service),
        match_service: Arc::new(match_service),
    };

    tokio::spawn(async move {
        let _ = run_client_server(client_service).await;
    });

    let app = AppService {};
    app.run(app_state).await;
    Ok(())
}
