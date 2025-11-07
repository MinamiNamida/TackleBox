use crate::{
    api::handler::{
        handle_delete_agent, handle_get_agent, handle_get_agents, handle_get_game_types,
        handle_get_match, handle_get_my_matches, handle_get_online_matches,
        handle_get_participants, handle_get_turns, handle_join_match, handle_login, handle_me,
        handle_new_agent, handle_new_match, handle_register, handle_update_agent,
    },
    core::{agents::AgentService, auth::AuthService, matches::MatchService},
};
use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::debug;

pub struct AppService {}

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub agent_service: Arc<AgentService>,
    pub match_service: Arc<MatchService>,
}

impl FromRef<AppState> for AuthState {
    fn from_ref(app_state: &AppState) -> AuthState {
        AuthState {
            auth_service: app_state.auth_service.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthState {
    pub auth_service: Arc<AuthService>,
}

#[derive(Clone)]
pub struct AgentState {
    pub auth_service: Arc<AuthService>,
    pub agent_service: Arc<AgentService>,
}

impl FromRef<AppState> for AgentState {
    fn from_ref(app_state: &AppState) -> Self {
        AgentState {
            auth_service: app_state.auth_service.clone(),
            agent_service: app_state.agent_service.clone(),
        }
    }
}

pub struct MatchState {
    pub match_service: Arc<MatchService>,
}

impl FromRef<AppState> for MatchState {
    fn from_ref(input: &AppState) -> Self {
        MatchState {
            match_service: input.match_service.clone(),
        }
    }
}

impl AppService {
    pub fn auth_router(&self) -> Router<AppState> {
        let router = Router::new()
            .route("/login", post(handle_login))
            .route("/register", post(handle_register))
            .route("/me", get(handle_me));
        router
    }

    pub fn agent_router(&self) -> Router<AppState> {
        let router = Router::new()
            .route("/new", post(handle_new_agent))
            .route("/delete", post(handle_delete_agent))
            .route("/update", post(handle_update_agent))
            .route("/get", post(handle_get_agent))
            .route("/agents", get(handle_get_agents));
        router
    }

    pub fn match_router(&self) -> Router<AppState> {
        let router = Router::new()
            .route("/new", post(handle_new_match))
            .route("/join", post(handle_join_match))
            .route("/get", post(handle_get_match))
            .route("/matches", get(handle_get_my_matches))
            .route("/turns", post(handle_get_turns))
            .route("/participants", post(handle_get_participants))
            .route("/gametypes", get(handle_get_game_types))
            .route("/search", get(handle_get_online_matches));
        router
    }

    pub fn api_router(&self) -> Router<AppState> {
        let router = Router::new()
            .nest("/auth", self.auth_router())
            .nest("/agent", self.agent_router())
            .nest("/match", self.match_router());
        router
    }

    pub async fn run(&self, app_state: AppState) {
        let router = Router::new()
            .nest("/api/v1", self.api_router())
            .with_state(app_state);

        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let listener = TcpListener::bind(addr).await.unwrap();
        debug!("App begin to serve at localhost:3000");
        axum::serve(listener, router).await.unwrap();
    }
}
