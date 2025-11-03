use std::{sync::Arc, time::Duration};

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    api::{
        app::{AppService, AppState},
        error::AppError,
        // ws::WsService,
    },
    core::{
        agents::AgentService,
        auth::{AuthConfig, AuthService},
        matches::MatchService,
        orchestrator::{run_client_server, OrchestratorService},
    },
    repo::{
        agents::AgentRepo, game_type::GameTypeRepo, matches::MatchRepo,
        participation::ParticipationRepo, turns::TurnRepo, users::UserRepo,
    },
};

pub mod api;
pub mod core;
pub mod repo;

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

    let orchestrator_service = Arc::new(
        OrchestratorService::new(
            agent_repo.clone(),
            match_repo.clone(),
            "http://localhost:50051".to_string(),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let match_service = MatchService::new(
        gametype_repo,
        user_repo,
        agent_repo,
        match_repo,
        turn_repo,
        participation_repo,
        orchestrator_service.clone(),
    );
    let app_state = AppState {
        agent_service: Arc::new(agent_service),
        auth_service: Arc::new(auth_service),
        orchestrator_service: orchestrator_service.clone(),
        match_service: Arc::new(match_service),
    };

    tokio::spawn(async move {
        let _ = run_client_server(orchestrator_service).await;
    });

    let app = AppService {};
    app.run(app_state).await;
    Ok(())
}
