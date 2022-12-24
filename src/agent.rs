use std::time::Duration;

use crate::http;
use anyhow::anyhow;
use tokio::{task, time};

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u32,
    pub url: String,
}

impl Agent {
    pub fn from_env() -> Vec<Agent> {
        let agent_urls = std::env::var("AGENT_URLS").expect("ENV 'AGENT_URLS' NOT FOUND!");
        let agent_replicas: Vec<(&str, u32)> = agent_urls
            .split(',')
            .filter_map(|f| f.split_once("|replicas="))
            .map(|(url, replicas)| (url, replicas.parse().unwrap_or(1)))
            .collect();

        let mut agents: Vec<Agent> = vec![];
        for agent_replica in agent_replicas {
            for i in 1..agent_replica.1 + 1 {
                agents.push(Agent {
                    id: i,
                    url: format!("{}-{}", agent_replica.0, i),
                });
            }
        }
        agents
    }

    // restart every agent one by one with 2 minutes interval
    pub async fn restart_all_agents(agents: Vec<Agent>) -> anyhow::Result<()> {
        for agent in agents.iter() {
            let url = format!("{}/shutdown", agent.url);
            http::hyper_delete(&url).await?;
            tracing::info!("restarting agent: {}", agent.id);
            tokio::time::sleep(Duration::from_secs(120)).await;
        }
        Ok(())
    }

    // every hour restart all agents periodically, with current config this supports about 25 agents
    pub fn restart_interval(agents: Vec<Agent>) {
        let forever = task::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(3600));

            loop {
                interval.tick().await;
                let _ = Self::restart_all_agents(agents.clone()).await;
            }
        });
    }

    pub async fn get_available(agents: Vec<Agent>) -> anyhow::Result<Agent> {
        let mut available_agents: Vec<&Agent> = vec![];

        for agent in agents.iter() {
            let is_busy = http::hyper_get(&format!("{}/is-busy", agent.url)).await?;
            if is_busy == "false" {
                available_agents.push(agent);
            }
        }

        if available_agents.is_empty() {
            return Err(anyhow!("NO AVAILABLE AGENTS"));
        }

        let index = rand::random::<usize>() % available_agents.len();
        let agent = available_agents[index];

        Ok(agent.clone())
    }
}
