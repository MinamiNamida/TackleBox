use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MatchMetadata {
    MatchMonitor { match_name: String },
    MatchPlayer { agent_name: String },
    None,
}
