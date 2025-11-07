use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;

/*
====================
Authenticated User,
User Payload and Response
====================
*/

#[derive(Deserialize, Serialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub token: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub token: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetUserResponse {
    pub user_id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

/*
====================
Agent Manager Payload
====================
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "agent_status", rename_all = "PascalCase")]
pub enum AgentStatus {
    Idle,
    Running,
    Ready,
    Decommissioned, //  被弃用的
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[sqlx(type_name = "agent_policy", rename_all = "PascalCase")]
pub enum AgentPolicy {
    Idle,
    AutoJoin,
    AutoNewAndJoin,
}

#[derive(Deserialize, Debug)]
pub struct GetAgentPayload {
    pub agent_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct GetAgentResponse {
    pub agent_id: Uuid,
    pub name: String,
    pub game_type_id: Uuid,
    pub game_type_name: String,
    pub owner_id: Uuid,
    pub owner_name: String,
    pub version: String,
    pub description: Option<String>,
    pub played_games: i32,
    pub won_games: i32,
    pub status: AgentStatus,
    pub policy: AgentPolicy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct NewAgentPayload {
    pub name: String,
    pub game_type_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub policy: AgentPolicy,
}

#[derive(Deserialize, Debug)]
pub struct UpdateAgentPayload {
    pub agent_id: Uuid,
    pub name: String,
    pub game_type_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub policy: AgentPolicy,
}

#[derive(Deserialize, Debug)]
pub struct DeleteAgentPayload {
    pub agent_id: Uuid,
}

/*
====================
Match Manager Payload
====================
*/

#[derive(Debug, Type, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[sqlx(type_name = "match_status", rename_all = "PascalCase")]
pub enum MatchStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
}

#[derive(Serialize, Deserialize)]
pub struct GetMatchPayload {
    pub match_id: Uuid,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct GetMatchResponse {
    pub match_id: Uuid,
    pub match_name: String,
    pub creater_id: Uuid,
    pub creater_name: String,
    pub winner_id: Option<Uuid>,
    pub winner_agent_name: Option<String>,
    pub game_type_id: Uuid,
    pub game_type_name: String,
    pub password: Option<String>,
    pub total_games: i32,
    pub max_slots: i32,
    pub min_slots: i32,
    pub status: MatchStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct GetOnlineMatchResponse {
    pub match_id: Uuid,
    pub match_name: String,
    pub creater_id: String,
    pub creater_name: String,
    pub game_type_id: Uuid,
    pub game_type_name: String,
    pub with_password: bool,
    pub max_slots: i32,
    pub min_slots: i32,
    pub current_slots: i64,
    pub status: MatchStatus,
    pub start_time: DateTime<Utc>,
    pub total_games: i32,
}

#[derive(Serialize, Deserialize)]
pub struct NewMatchPayload {
    pub name: String,
    pub game_type_id: Uuid,
    pub total_games: i32,
    pub with_agent_ids: Vec<Uuid>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateMatchPayload {
    pub name: String,
    pub game_type_id: Uuid,
    pub total_games: i32,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JoinMatchPayload {
    pub match_id: Uuid,
    pub agent_ids: Vec<Uuid>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LeaveMatchPayload {
    pub match_id: Uuid,
    pub agent_ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct GetMatchLogsPayload {
    pub match_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct TurnLogResponse {
    pub turn_id: Uuid,
    pub match_id: Uuid,
    pub log: Value,
    pub i_turn: i32,
    pub score_deltas: Value,
}

#[derive(Serialize, Deserialize)]
pub struct GetParticipantsPayload {
    pub match_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct GetParticipantsResponse {
    pub match_id: Uuid,
    pub match_name: String,
    pub agent_id: Uuid,
    pub agent_name: String,
}

/*
====================
GameType Payload
====================
*/

#[derive(Serialize, Deserialize)]
pub struct GetGameTypeResponse {
    pub game_type_id: Uuid,
    pub name: String,
    pub sponsor: String,
    pub description: Option<String>,
    pub min_slots: i32,
    pub max_slots: i32,
}

/*
====================
Stats Payload
====================
*/

#[derive(Serialize, Deserialize)]
pub struct GetStatsResponse {
    pub game_type_id: Uuid,
    pub game_type_name: String,
    pub agent_id: Uuid,
    pub agent_name: String,
    pub rank: i32,
    pub updated_time: DateTime<Utc>,
}
