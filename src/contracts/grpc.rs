use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MatchMetadata {
    MatchMonitor { match_id: Uuid },
    MatchPlayer { agent_id: Uuid },
    None,
}
