use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub token: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetUserResponse {
    pub user_id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

/*
====================
Agent Manager Payload
====================
*/

#[derive(Debug, Deserialize, Serialize)]
pub enum AgentStatus {
    Idle,
    Pending,
    Running,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AgentPolicy {
    AutoJoin,
    AutoNewAndJoin,
    Idle,
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
    pub version: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub played_games: i32,
    pub won_games: i32,
    pub status: AgentStatus,
    pub policy: AgentPolicy,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct NewAgentPayload {
    pub name: String,
    pub game_type_id: Uuid,
    pub version: String,
    pub description: Option<String>,
    pub policy: AgentPolicy,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct NewAgentResponse {
    pub agent_id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct UpdateAgentPayload {
    pub agent_id: Uuid,
    pub name: String,
    pub game_type: String,
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

#[derive(Serialize, Deserialize)]
pub struct GetMatchPayload {
    pub match_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct GetMatchResponse {
    pub match_id: Uuid,
    pub match_name: String,
    pub game_type_id: Uuid,
    pub game_type: String,
    pub creater_id: Uuid,
    pub creater_name: String,
    pub total_games: i32,
    pub winner_agent_id: Option<String>,
    pub winner_agent_name: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetOnlineMatchResponse {
    pub match_id: Uuid,
    pub match_name: String,
    pub creater_id: String,
    pub creater_name: String,
    pub game_type_id: Uuid,
    pub game_type: String,
    pub with_password: bool,
    pub slots: i32,
    pub status: String,
    pub start_time: String,
    pub total_games: i32,
}

#[derive(Serialize, Deserialize)]
pub struct NewMatchPayload {
    pub name: String,
    pub game_type: String,
    pub total_games: i32,
    pub with_agent_names: Vec<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateMatchPayload {
    pub name: String,
    pub game_type: String,
    pub total_games: i32,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JoinMatchPayload {
    pub match_id: Uuid,
    pub agent_ids: Vec<Uuid>,
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
pub struct GetPariticipantsPayload {
    pub match_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct GetParticipantsResponse {
    pub match_id: Uuid,
    pub agent_id: Uuid,
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
    pub updated_time: String,
}
