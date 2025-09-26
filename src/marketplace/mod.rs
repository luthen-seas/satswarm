// src/marketplace/mod.rs
use serde::{Deserialize, Serialize};
use nostr::{Event, EventBuilder, Keys, Kind, Tag, Filter, Timestamp};
use nostr_sdk::{Client, RelayUrl};
use std::collections::HashMap;
use uuid::Uuid;
use tokio::time::{Duration, timeout};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    pub pubkey: String,
    pub name: String,
    pub skills: Vec<String>,
    pub hourly_rate_sats: u64,
    pub reputation_score: f64,
    pub completed_tasks: u32,
    pub success_rate: f64,
    pub last_active: u64,
    pub specializations: Vec<String>,
    pub compute_resources: ComputeResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResources {
    pub cpu_cores: u32,
    pub gpu_available: bool,
    pub ram_gb: u32,
    pub storage_gb: u32,
    pub bandwidth_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBid {
    pub bid_id: String,
    pub task_id: String,
    pub specialist_pubkey: String,
    pub quoted_price_sats: u64,
    pub estimated_completion_time: u32, // minutes
    pub proposal: String,
    pub stake_amount: u64, // Collateral in sats
    pub expires_at: u64,
    pub metadata: BidMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidMetadata {
    pub confidence_level: f64,
    pub similar_tasks_completed: u32,
    pub compute_requirements: ComputeResources,
    pub delivery_format: String,
    pub includes_validation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub requester_pubkey: String,
    pub description: String,
    pub required_skills: Vec<String>,
    pub max_budget_sats: u64,
    pub deadline_minutes: u32,
    pub quality_requirements: QualityRequirements,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub min_reputation: f64,
    pub min_success_rate: f64,
    pub requires_validation: bool,
    pub preferred_specialists: Vec<String>,
    pub blacklisted_specialists: Vec<String>,
}

pub struct MarketplaceClient {
    nostr_client: Client,
    local_relay_url: String,
    specialist_profiles: HashMap<String, SpecialistProfile>,
    active_bids: HashMap<String, Vec<TaskBid>>,
}

impl MarketplaceClient {
    pub async fn new(local_relay_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new(&Keys::generate());
        client.add_relay(local_relay_url).await?;
        client.connect().await;

        Ok(Self {
            nostr_client: client,
            local_relay_url: local_relay_url.to_string(),
            specialist_profiles: HashMap::new(),
            active_bids: HashMap::new(),
        })
    }

    // Broadcast task request to specialists
    pub async fn broadcast_task_request(
        &mut self,
        task_request: &TaskRequest,
        keys: &Keys,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string(task_request)?;
        let tags = vec![
            Tag::generic(nostr::TagKind::Custom("task_type".into()), vec!["request"]),
            Tag::generic(nostr::TagKind::Custom("skills".into()), task_request.required_skills.clone()),
            Tag::generic(nostr::TagKind::Custom("max_budget".into()), vec![&task_request.max_budget_sats.to_string()]),
            Tag::generic(nostr::TagKind::Custom("deadline".into()), vec![&task_request.deadline_minutes.to_string()]),
        ];

        let event = EventBuilder::new(Kind::Custom(30002), &content, tags)
            .sign_with_keys(keys)?;

        self.nostr_client.send_event(event).await?;
        println!("Task request broadcast: {}", task_request.task_id);
        Ok(())
    }

    // Listen for incoming bids
    pub async fn collect_bids(
        &mut self,
        task_id: &str,
        timeout_seconds: u64,
    ) -> Result<Vec<TaskBid>, Box<dyn std::error::Error>> {
        let filter = Filter::new()
            .kind(Kind::Custom(30003))
            .custom_tag(nostr::SingleLetterTag::lowercase(nostr::Alphabet::T), vec![task_id]);

        let timeout_duration = Duration::from_secs(timeout_seconds);
        let mut collected_bids = Vec::new();

        println!("Collecting bids for task {} (timeout: {}s)...", task_id, timeout_seconds);

        let result = timeout(timeout_duration, async {
            let events = self.nostr_client.get_events_of(vec![filter], None).await?;
            
            for event in events {
                if let Ok(bid) = self.parse_bid_event(&event) {
                    if self.validate_bid(&bid) {
                        collected_bids.push(bid);
                    }
                }
            }
            
            Ok::<_, Box<dyn std::error::Error>>(collected_bids)
        }).await;

        match result {
            Ok(bids) => Ok(bids?),
            Err(_) => {
                println!("Bid collection timed out, returning {} collected bids", collected_bids.len());
                Ok(collected_bids)
            }
        }
    }

    // Simulate specialist bids for MVP demo
    pub async fn simulate_bids(&self, task_request: &TaskRequest) -> Vec<TaskBid> {
        let mock_specialists = self.get_mock_specialists();
        let mut bids = Vec::new();

        for specialist in mock_specialists {
            // Check if specialist matches required skills
            let skill_match = task_request.required_skills.iter()
                .any(|required_skill| specialist.skills.contains(required_skill));

            if skill_match && specialist.reputation_score >= task_request.quality_requirements.min_reputation {
                let bid = self.generate_mock_bid(&specialist, task_request);
                bids.push(bid);
            }
        }

        // Sort by price and reputation
        bids.sort_by(|a, b| {
            let a_score = self.calculate_bid_score(a);
            let b_score = self.calculate_bid_score(b);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        bids
    }

    fn get_mock_specialists(&self) -> Vec<SpecialistProfile> {
        vec![
            SpecialistProfile {
                pubkey: "specialist_1_pubkey".to_string(),
                name: "BioinformaticsBot".to_string(),
                skills: vec!["bioinformatics".to_string(), "sequence_analysis".to_string(), "alphafold".to_string()],
                hourly_rate_sats: 150,
                reputation_score: 4.8,
                completed_tasks: 342,
                success_rate: 0.97,
                last_active: chrono::Utc::now().timestamp() as u64,
                specializations: vec!["protein_folding".to_string(), "molecular_dynamics".to_string()],
                compute_resources: ComputeResources {
                    cpu_cores: 32,
                    gpu_available: true,
                    ram_gb: 128,
                    storage_gb: 2000,
                    bandwidth_mbps: 1000,
                },
            },
            SpecialistProfile {
                pubkey: "specialist_2_pubkey".to_string(),
                name: "QuantMaster".to_string(),
                skills: vec!["quantitative_analysis".to_string(), "risk_modeling".to_string(), "machine_learning".to_string()],
                hourly_rate_sats: 200,
                reputation_score: 4.9,
                completed_tasks: 198,
                success_rate: 0.95,
                last_active: chrono::Utc::now().timestamp() as u64,
                specializations: vec!["portfolio_optimization".to_string(), "derivatives_pricing".to_string()],
                compute_resources: ComputeResources {
                    cpu_cores: 16,
                    gpu_available: false,
                    ram_gb: 64,
                    storage_gb: 1000,
                    bandwidth_mbps: 500,
                },
            },
            SpecialistProfile {
                pubkey: "specialist_3_pubkey".to_string(),
                name: "MLNinja".to_string(),
                skills: vec!["machine_learning".to_string(), "deep_learning".to_string(), "gpu_computing".to_string()],
                hourly_rate_sats: 180,
                reputation_score: 4.6,
                completed_tasks: 267,
                success_rate: 0.93,
                last_active: chrono::Utc::now().timestamp() as u64,
                specializations: vec!["computer_vision".to_string(), "nlp".to_string()],
                compute_resources: ComputeResources {
                    cpu_cores: 24,
                    gpu_available: true,
                    ram_gb: 256,
                    storage_gb: 5000,
                    bandwidth_mbps: 2000,
                },
            },
        ]
    }

    fn generate_mock_bid(&self, specialist: &SpecialistProfile, task: &TaskRequest) -> TaskBid {
        // Calculate bid price based on specialist's rate and task complexity
        let base_price = specialist.hourly_rate_sats * (task.deadline_minutes as u64 / 60).max(1);
        let complexity_multiplier = if task.required_skills.len() > 2 { 1.3 } else { 1.0 };
        let quoted_price = (base_price as f64 * complexity_multiplier) as u64;

        // Ensure bid is within budget
        let final_price = quoted_price.min(task.max_budget_sats);

        TaskBid {
            bid_id: Uuid::new_v4().to_string(),
            task_id: task.task_id.clone(),
            specialist_pubkey: specialist.pubkey.clone(),
            quoted_price_sats: final_price,
            estimated_completion_time: (task.deadline_minutes as f64 * 0.8) as u32,
            proposal: format!("I can complete this {} task using my expertise in {}. My approach includes: 1) Initial analysis, 2) Core computation, 3) Results validation.", 
                            task.description, 
                            specialist.specializations.join(", ")),
            stake_amount: final_price / 10, // 10% stake
            expires_at: chrono::Utc::now().timestamp() as u64 + 3600, // 1 hour expiry
            metadata: BidMetadata {
                confidence_level: specialist.success_rate,
                similar_tasks_completed: specialist.completed_tasks / 10,
                compute_requirements: specialist.compute_resources.clone(),
                delivery_format: "JSON + PDF report".to_string(),
                includes_validation: task.quality_requirements.requires_validation,
            },
        }
    }

    fn calculate_bid_score(&self, bid: &TaskBid) -> f64 {
        // Multi-factor scoring: price (40%), reputation (30%), completion time (20%), stake (10%)
        let price_score = 1.0 - (bid.quoted_price_sats as f64 / 10000.0).min(1.0);
        let confidence_score = bid.metadata.confidence_level;
        let time_score = 1.0 - (bid.estimated_completion_time as f64 / 1440.0).min(1.0); // Normalize to 24 hours
        let stake_score = (bid.stake_amount as f64 / bid.quoted_price_sats as f64).min(0.2) * 5.0;

        price_score * 0.4 + confidence_score * 0.3 + time_score * 0.2 + stake_score * 0.1
    }

    fn validate_bid(&self, bid: &TaskBid) -> bool {
        // Basic validation
        bid.quoted_price_sats > 0 
            && bid.estimated_completion_time > 0 
            && bid.expires_at > chrono::Utc::now().timestamp() as u64
            && !bid.specialist_pubkey.is_empty()
    }

    fn parse_bid_event(&self, event: &Event) -> Result<TaskBid, Box<dyn std::error::Error>> {
        let bid: TaskBid = serde_json::from_str(&event.content)?;
        Ok(bid)
    }

    pub async fn select_winning_bid(&self, bids: Vec<TaskBid>) -> Option<TaskBid> {
        if bids.is_empty() {
            return None;
        }

        // For MVP, select the highest-scoring bid
        let mut scored_bids: Vec<_> = bids.into_iter()
            .map(|bid| {
                let score = self.calculate_bid_score(&bid);
                (bid, score)
            })
            .collect();

        scored_bids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Some(scored_bids.into_iter().next()?.0)
    }

    pub fn print_bid_summary(&self, bids: &[TaskBid]) {
        println!("\n=== Bid Summary ===");
        for (i, bid) in bids.iter().enumerate() {
            let score = self.calculate_bid_score(bid);
            println!("{}. Specialist: {} | Price: {} sats | Time: {} min | Score: {:.3}", 
                    i + 1, 
                    bid.specialist_pubkey, 
                    bid.quoted_price_sats, 
                    bid.estimated_completion_time,
                    score);
        }
        println!("==================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_scoring() {
        let client = MarketplaceClient {
            nostr_client: Client::new(&Keys::generate()),
            local_relay_url: "ws://localhost:8080".to_string(),
            specialist_profiles: HashMap::new(),
            active_bids: HashMap::new(),
        };

        let bid = TaskBid {
            bid_id: "test".to_string(),
            task_id: "test".to_string(),
            specialist_pubkey: "test".to_string(),
            quoted_price_sats: 500,
            estimated_completion_time: 120,
            proposal: "test".to_string(),
            stake_amount: 50,
            expires_at: chrono::Utc::now().timestamp() as u64 + 3600,
            metadata: BidMetadata {
                confidence_level: 0.9,
                similar_tasks_completed: 10,
                compute_requirements: ComputeResources {
                    cpu_cores: 8,
                    gpu_available: true,
                    ram_gb: 32,
                    storage_gb: 500,
                    bandwidth_mbps: 100,
                },
                delivery_format: "JSON".to_string(),
                includes_validation: true,
            },
        };

        let score = client.calculate_bid_score(&bid);
        assert!(score > 0.0 && score <= 1.0);
    }
}