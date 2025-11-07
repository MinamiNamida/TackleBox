use crate::{
    api::error::AppError,
    repo::{
        agents::{AgentRepo, GetRankableAgentDTO},
        stats::{StatsRepo, UpdateStatsDTO},
    },
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time;
use tracing::{info, warn};
use uuid::Uuid;

struct Repos {
    agent_repo: Arc<AgentRepo>,
    stats_repo: Arc<StatsRepo>,
}

pub struct StatsService {
    repos: Repos,
}

impl StatsService {
    pub fn new(agent_repo: Arc<AgentRepo>, stats_repo: Arc<StatsRepo>) -> Self {
        Self {
            repos: Repos {
                agent_repo,
                stats_repo,
            },
        }
    }
    pub async fn update_stats(&self) -> Result<(), AppError> {
        info!("Starting stats update task...");
        let agents = self.repos.agent_repo.rankable_agents().await?;
        let mut ranks_to_upsert: Vec<(Uuid, Uuid, i32)> = Vec::new();
        let mut grouped_data: HashMap<Uuid, Vec<GetRankableAgentDTO>> = HashMap::new();
        for row in agents {
            if row.played_games > 0 {
                grouped_data.entry(row.game_type_id).or_default().push(row);
            }
        }
        for (_game_type_id, mut agents) in grouped_data {
            agents.sort_by(|a, b| {
                let win_rate_a = a.won_games as f64 / a.played_games as f64;
                let win_rate_b = b.won_games as f64 / b.played_games as f64;

                win_rate_b
                    .partial_cmp(&win_rate_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(b.played_games.cmp(&a.played_games))
            });

            for (index, agent) in agents.into_iter().enumerate() {
                let rank = (index + 1) as i32;
                ranks_to_upsert.push((agent.agent_id, agent.game_type_id, rank));
            }
        }

        let (agent_ids, game_type_ids, new_ranks): (Vec<Uuid>, Vec<Uuid>, Vec<i32>) =
            ranks_to_upsert
                .into_iter()
                .map(|(a, g, r)| (a, g, r))
                .collect();

        info!(
            "Calculated {} new rank entries. Upserting...",
            agent_ids.len()
        );
        let data = UpdateStatsDTO {
            agent_ids,
            game_type_ids,
            new_ranks,
        };
        self.repos.stats_repo.update_stats(data).await?;

        info!("Stats update completed successfully.");
        Ok(())
    }

    pub async fn run(&self) -> Result<(), AppError> {
        let mut interval = time::interval(Duration::from_secs(60 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            info!("--- STATS UPDATE: Starting scheduled rank calculation. ---");
            let result = self.update_stats().await;
            match result {
                Ok(_) => info!("--- STATS UPDATE: Successfully completed rank update. ---"),
                Err(e) => warn!("--- STATS ERROR: Failed to update stats: {:?} ---", e),
            }
        }
    }
}
