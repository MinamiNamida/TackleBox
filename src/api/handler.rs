use crate::{
    api::{
        app::{AgentState, AuthState, MatchState},
        error::AppError,
        extractor::{generate_jwt, AuthenticatedUser},
    },
    repo::users::GetUserDTO,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tackle_box::contracts::payloads_v1::{
    DeleteAgentPayload, GetAgentPayload, GetMatchLogsPayload, GetMatchPayload,
    GetPariticipantsPayload, GetUserResponse, JoinMatchPayload, LeaveMatchPayload, LoginPayload,
    LoginResponse, NewAgentPayload, NewMatchPayload, RegisterPayload, RegisterResponse,
    UpdateAgentPayload,
};
/*
====================
Handle User Profile
====================
*/

pub async fn handle_login(
    State(state): State<AuthState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let LoginPayload { username, password } = payload;
    let user_id = state.auth_service.login(&username, &password).await?;
    let jwt_token = generate_jwt(&username, user_id).await?;
    let resp = LoginResponse {
        token: jwt_token,
        token_type: "Bearer".to_string(),
    };
    Ok((StatusCode::OK, Json(json!(resp))))
}

pub async fn handle_register(
    State(state): State<AuthState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    let RegisterPayload {
        username,
        password,
        email,
    } = payload;
    let user_id = state.auth_service.register(&username, &password).await?;
    let jwt_token = generate_jwt(&username, user_id).await?;
    let resp = RegisterResponse {
        token: jwt_token,
        token_type: "Bearer".to_string(),
    };
    Ok((StatusCode::OK, Json(json!(resp))))
}

pub async fn handle_me(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AuthState>,
) -> Result<impl IntoResponse, AppError> {
    let me = state.auth_service.me(user_id).await?;
    let GetUserDTO {
        username,
        created_at,
        ..
    } = me;
    let user = GetUserResponse {
        username,
        created_at,
    };
    Ok((StatusCode::OK, Json(json!(user))))
}

/*
====================
Agent Manager Handler
====================
*/

pub async fn handle_get_agent(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AgentState>,
    Json(payload): Json<GetAgentPayload>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state.agent_service.get_agent(user_id, payload.name).await?;
    Ok((StatusCode::OK, Json(json!(agent))))
}

pub async fn handle_new_agent(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AgentState>,
    Json(payload): Json<NewAgentPayload>,
) -> Result<impl IntoResponse, AppError> {
    let _ = state.agent_service.new_agent(user_id, payload).await?;
    Ok(StatusCode::OK)
}

pub async fn handle_update_agent(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AgentState>,
    Json(payload): Json<UpdateAgentPayload>,
) -> Result<impl IntoResponse, AppError> {
    state.agent_service.update_agent(user_id, payload).await?;
    Ok(StatusCode::OK)
}

pub async fn handle_delete_agent(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AgentState>,
    Json(payload): Json<DeleteAgentPayload>,
) -> Result<impl IntoResponse, AppError> {
    state
        .agent_service
        .delete_agent(user_id, payload.name)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn handle_get_agents(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<AgentState>,
) -> Result<impl IntoResponse, AppError> {
    let agents = state.agent_service.get_agents_by_owner_id(user_id).await?;
    Ok((StatusCode::OK, Json(json!(agents))))
}

/*
====================
Match Manager Handler
====================
*/

pub async fn handle_get_match(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
    Json(payload): Json<GetMatchPayload>,
) -> Result<impl IntoResponse, AppError> {
    let one_match = state.match_service.get_match(&payload.name).await?;
    Ok((StatusCode::OK, Json(json!(one_match))))
}

pub async fn handle_get_my_matches(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
) -> Result<impl IntoResponse, AppError> {
    let matches = state.match_service.get_my_matches(user_id).await?;
    Ok((StatusCode::OK, Json(json!(matches))))
}

pub async fn handle_new_match(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
    Json(payload): Json<NewMatchPayload>,
) -> Result<impl IntoResponse, AppError> {
    let _ = state.match_service.new_match(user_id, payload).await?;
    Ok(StatusCode::OK)
}

pub async fn handle_join_match(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
    Json(payload): Json<JoinMatchPayload>,
) -> Result<impl IntoResponse, AppError> {
    state
        .match_service
        .join_match(user_id, &payload.match_name, &payload.agent_name)
        .await?;
    Ok(StatusCode::OK)
}

// pub async fn handle_leave_match(
//     AuthenticatedUser { user_id }: AuthenticatedUser,
//     State(state): State<MatchState>,
//     Json(payload): Json<LeaveMatchPayload>,
// ) -> Result<impl IntoResponse, AppError> {
//     state
//         .match_service
//         .leave_match(user_id, payload.agent_name)
//         .await?;
//     Ok(StatusCode::OK)
// }

pub async fn handle_get_online_matches(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
) -> Result<impl IntoResponse, AppError> {
    let matches = state.match_service.get_online_matches(user_id).await?;
    Ok((StatusCode::OK, Json(json!(matches))))
}

pub async fn handle_get_turns(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
    Json(payload): Json<GetMatchLogsPayload>,
) -> Result<impl IntoResponse, AppError> {
    let turns = state
        .match_service
        .get_match_logs(user_id, &payload.match_name)
        .await?;
    Ok((StatusCode::OK, Json(json!(turns))))
}

pub async fn handle_get_participants(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
    Json(payload): Json<GetPariticipantsPayload>,
) -> Result<impl IntoResponse, AppError> {
    let parts = state
        .match_service
        .get_participants(user_id, &payload.match_name)
        .await?;
    Ok((StatusCode::OK, Json(json!(parts))))
}

/*
====================
GameType Handler
====================
*/

pub async fn handle_get_game_types(
    AuthenticatedUser { user_id }: AuthenticatedUser,
    State(state): State<MatchState>,
) -> Result<impl IntoResponse, AppError> {
    let gametypes = state.match_service.get_gametypes().await?;
    Ok((StatusCode::OK, Json(json!(gametypes))))
}
