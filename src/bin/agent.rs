// src/agent/cli.rs
use chrono::Utc;
use clap::{App, Arg, SubCommand};
use std::io::{self, Write};
use tokio;
use uuid::Uuid;

// Import our modules
use crate::decomposition::{TaskDecomposer, TaskStatus};
use crate::marketplace::{MarketplaceClient, QualityRequirements, TaskRequest};
use crate::observability::{ObservabilityDashboard, TaskEventType};
use crate::payments::{EscrowConditions, SatswarmPaymentSystem};
use crate::privacy::{AccessControl, ContentType, PrivacyLevel, PrivacyManager};
use crate::reputation::ReputationManager;
use nostr::Keys;

pub struct GeneralAgent {
    keys: Keys,
    task_decomposer: TaskDecomposer,
    marketplace_client: MarketplaceClient,
    payment_system: SatswarmPaymentSystem,
    reputation_manager: ReputationManager,
    privacy_manager: PrivacyManager,
    dashboard: ObservabilityDashboard,
}

impl GeneralAgent {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let keys = Keys::generate();
        let marketplace_client = MarketplaceClient::new("ws://localhost:8080").await?;

        Ok(Self {
            keys,
            task_decomposer: TaskDecomposer::new(),
            marketplace_client,
            payment_system: SatswarmPaymentSystem::new(),
            reputation_manager: ReputationManager::new(),
            privacy_manager: PrivacyManager::new(),
            dashboard: ObservabilityDashboard::new(),
        })
    }

    pub async fn run_cli(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Satswarm General Agent CLI");
        println!("Agent Public Key: {}", self.keys.public_key());
        println!("Type 'help' for available commands or 'demo' for full workflow demo\n");

        loop {
            print!("satswarm> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "help" => self.print_help(),
                "demo" => self.run_demo().await?,
                "status" => self.show_status(),
                "dashboard" => self.dashboard.print_dashboard(),
                "exit" | "quit" => {
                    println!("👋 Goodbye!");
                    break;
                }
                _ if input.starts_with("task ") => {
                    let request = input.strip_prefix("task ").unwrap();
                    self.process_task_request(request).await?;
                }
                _ if input.starts_with("balance") => {
                    self.show_balance();
                }
                _ => {
                    println!(
                        "Unknown command: '{}'. Type 'help' for available commands.",
                        input
                    );
                }
            }
        }

        Ok(())
    }

    fn print_help(&self) {
        println!("\n=== SATSWARM GENERAL AGENT COMMANDS ===");
        println!("task <description>  - Process a new task request");
        println!("demo               - Run complete workflow demonstration");
        println!("status             - Show agent status");
        println!("balance            - Show payment balances");
        println!("dashboard          - Show system dashboard");
        println!("help               - Show this help message");
        println!("exit/quit          - Exit the application");
        println!("=======================================\n");
    }

    async fn run_demo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🎬 Starting Satswarm MVP Demo...");
        println!("==================================");

        // Demo task request
        let demo_request = "Analyze the protein structure of BRCA1 for drug binding sites";

        println!("\n1️⃣  TASK REQUEST");
        println!("User request: '{}'", demo_request);

        // Step 1: Task Decomposition
        println!("\n2️⃣  TASK DECOMPOSITION");
        let task_tree = self.task_decomposer.decompose(demo_request)?;

        println!(
            "✅ Task decomposed into {} subtasks:",
            task_tree.root.subtasks.len()
        );
        for (i, subtask) in task_tree.root.subtasks.iter().enumerate() {
            println!(
                "  {}. {} ({} sats)",
                i + 1,
                subtask.description,
                subtask.estimated_sats
            );
        }
        println!(
            "Total estimated cost: {} sats",
            task_tree.total_estimated_cost
        );

        // Update dashboard
        self.dashboard.record_task_event(
            &task_tree.root.id,
            TaskEventType::TaskCreated,
            &self.keys.public_key().to_string(),
            std::collections::HashMap::new(),
        );

        // Step 2: Marketplace Broadcast
        println!("\n3️⃣  MARKETPLACE BROADCAST");
        let task_request = TaskRequest {
            task_id: task_tree.root.id.clone(),
            requester_pubkey: self.keys.public_key().to_string(),
            description: demo_request.to_string(),
            required_skills: vec!["bioinformatics".to_string(), "protein_analysis".to_string()],
            max_budget_sats: task_tree.total_estimated_cost,
            deadline_minutes: 120,
            quality_requirements: QualityRequirements {
                min_reputation: 4.0,
                min_success_rate: 0.9,
                requires_validation: true,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        };

        self.marketplace_client
            .broadcast_task_request(&task_request, &self.keys)
            .await?;

        self.dashboard.record_task_event(
            &task_tree.root.id,
            TaskEventType::TaskBroadcast,
            &self.keys.public_key().to_string(),
            std::collections::HashMap::new(),
        );

        println!("✅ Task broadcast to marketplace");

        // Step 3: Collect Bids (Simulated)
        println!("\n4️⃣  COLLECTING BIDS");
        println!("Waiting for specialist bids... (simulating)");

        let bids = self.marketplace_client.simulate_bids(&task_request).await;
        println!("✅ Received {} bids from specialists", bids.len());

        self.marketplace_client.print_bid_summary(&bids);

        for bid in &bids {
            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::BidReceived,
                &bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );
        }

        // Step 4: Select Winning Bid
        println!("\n5️⃣  BID SELECTION");
        if let Some(winning_bid) = self.marketplace_client.select_winning_bid(bids).await {
            println!("🏆 Selected winning bid:");
            println!("   Specialist: {}", &winning_bid.specialist_pubkey[..16]);
            println!("   Price: {} sats", winning_bid.quoted_price_sats);
            println!(
                "   Estimated time: {} minutes",
                winning_bid.estimated_completion_time
            );

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::BidSelected,
                &winning_bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );

            // Step 5: Create Payment Escrow
            println!("\n6️⃣  PAYMENT ESCROW");
            let escrow_conditions = EscrowConditions {
                requires_verification: true,
                min_quality_rating: 4.0,
                deadline_unix: Utc::now().timestamp() as u64 + 7200, // 2 hours
                penalty_percentage: 20,
                auto_release_hours: 24,
            };

            let escrow = self
                .payment_system
                .create_escrow(
                    &task_tree.root.id,
                    &task_request.requester_pubkey,
                    &winning_bid.specialist_pubkey,
                    winning_bid.quoted_price_sats,
                    escrow_conditions,
                )
                .await?;

            println!("✅ Escrow created: {} sats locked", escrow.amount_sats);

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::TaskAssigned,
                &winning_bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );

            // Step 6: Simulate Task Execution
            println!("\n7️⃣  TASK EXECUTION");
            println!("Specialist working on task... (simulating)");

            // Simulate work time
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::WorkSubmitted,
                &winning_bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );

            println!("✅ Work completed and submitted by specialist");

            // Step 7: Mock Verification
            println!("\n8️⃣  VERIFICATION");
            println!("Verifying output... (mocking BitVM verification)");

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::VerificationStarted,
                "verifier_node",
                std::collections::HashMap::new(),
            );

            // Simulate verification time
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            let quality_rating = 4.6; // Mock high-quality result
            println!(
                "✅ Verification completed - Quality rating: {}/5.0",
                quality_rating
            );

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::VerificationCompleted,
                "verifier_node",
                std::collections::HashMap::new(),
            );

            // Step 8: Release Payment
            println!("\n9️⃣  PAYMENT RELEASE");
            self.payment_system
                .release_escrow(&escrow.escrow_id, quality_rating)
                .await?;

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::PaymentReleased,
                &winning_bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );

            // Step 9: Update Reputation
            println!("\n🔟 REPUTATION UPDATE");
            let rating = crate::reputation::TaskRating {
                rating_id: Uuid::new_v4().to_string(),
                task_id: task_tree.root.id.clone(),
                specialist_pubkey: winning_bid.specialist_pubkey.clone(),
                requester_pubkey: task_request.requester_pubkey.clone(),
                quality_score: quality_rating,
                timeliness_score: 4.8,
                communication_score: 4.5,
                would_work_again: true,
                comment: "Excellent protein analysis with detailed binding site identification"
                    .to_string(),
                verified_completion: true,
                created_at: Utc::now().timestamp() as u64,
                signature: "mock_signature".to_string(),
            };

            self.reputation_manager.submit_rating(rating, &self.keys)?;
            println!("✅ Reputation updated for specialist");

            self.dashboard.record_task_event(
                &task_tree.root.id,
                TaskEventType::TaskCompleted,
                &winning_bid.specialist_pubkey,
                std::collections::HashMap::new(),
            );

            // Step 10: Economic Loop Update & Deflationary Distribution
            println!("\n🎁 ECONOMIC REWARDS");
            let distributions = self
                .payment_system
                .distribute_deflationary_rewards()
                .await?;

            if !distributions.is_empty() {
                println!("✅ Deflationary rewards distributed:");
                for (pubkey, amount) in distributions {
                    println!("   {}: {} sats", &pubkey[..16], amount);
                }
            }

            // Final Status
            println!("\n✨ DEMO COMPLETED SUCCESSFULLY!");
            println!("=====================================");
            self.payment_system.print_economic_status();

            // Update dashboard metrics
            self.dashboard
                .update_metrics(5, 0, 1, winning_bid.quoted_price_sats, 0);
        } else {
            println!("❌ No suitable bids received");
        }

        Ok(())
    }

    async fn process_task_request(
        &mut self,
        request: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 Processing task request: '{}'", request);

        // Decompose task
        let task_tree = self.task_decomposer.decompose(request)?;
        println!(
            "✅ Task decomposed into {} subtasks",
            task_tree.root.subtasks.len()
        );

        // Create marketplace request
        let task_request = TaskRequest {
            task_id: task_tree.root.id.clone(),
            requester_pubkey: self.keys.public_key().to_string(),
            description: request.to_string(),
            required_skills: task_tree
                .root
                .subtasks
                .iter()
                .flat_map(|st| st.required_skills.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
            max_budget_sats: task_tree.total_estimated_cost,
            deadline_minutes: 120,
            quality_requirements: QualityRequirements {
                min_reputation: 4.0,
                min_success_rate: 0.9,
                requires_validation: true,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        };

        // Broadcast to marketplace
        self.marketplace_client
            .broadcast_task_request(&task_request, &self.keys)
            .await?;
        println!("✅ Task broadcast to marketplace");

        // Collect and display simulated bids
        let bids = self.marketplace_client.simulate_bids(&task_request).await;
        if !bids.is_empty() {
            println!("📥 Received {} bids:", bids.len());
            self.marketplace_client.print_bid_summary(&bids);
        } else {
            println!("❌ No bids received");
        }

        Ok(())
    }

    fn show_status(&self) {
        println!("\n=== AGENT STATUS ===");
        println!("Public Key: {}", self.keys.public_key());
        println!("Connected to relay: ws://localhost:8080");

        let payment_status = self
            .payment_system
            .get_payment_status(&self.keys.public_key().to_string());
        println!("Balance: {} sats", payment_status.balance_sats);

        if let Some(tier) = payment_status.economic_tier {
            println!("Economic Tier: {:?}", tier);
            println!(
                "Visibility Multiplier: {:.1}x",
                payment_status.visibility_multiplier
            );
        }

        println!("Active Escrows: {}", payment_status.active_escrows);
        println!("Pending Transfers: {}", payment_status.pending_transfers);
        println!("===================\n");
    }

    fn show_balance(&self) {
        println!("\n💰 PAYMENT BALANCE");
        let status = self
            .payment_system
            .get_payment_status(&self.keys.public_key().to_string());
        println!("Current Balance: {} sats", status.balance_sats);

        if let Some(tier) = &status.economic_tier {
            println!("Performance Tier: {:?}", tier);
            println!("Rate Multiplier: {:.2}x", status.visibility_multiplier);
        }
        println!();
    }
}

// Main CLI application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let matches = App::new("Satswarm General Agent")
        .version("0.1.0")
        .author("Satswarm Team")
        .about("AI Agent Marketplace on Bitcoin & NOSTR")
        .arg(
            Arg::with_name("relay")
                .long("relay")
                .value_name("URL")
                .help("NOSTR relay URL")
                .takes_value(true)
                .default_value("ws://localhost:8080"),
        )
        .arg(
            Arg::with_name("mint")
                .long("mint")
                .value_name("URL")
                .help("Cashu mint URL")
                .takes_value(true)
                .default_value("http://localhost:3338"),
        )
        .subcommand(SubCommand::with_name("interactive").about("Start interactive CLI mode"))
        .subcommand(SubCommand::with_name("demo").about("Run the full workflow demo"))
        .subcommand(
            SubCommand::with_name("task")
                .about("Process a single task")
                .arg(
                    Arg::with_name("description")
                        .help("Task description")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    println!("🚀 Initializing Satswarm General Agent...");

    // Create agent instance
    let mut agent = GeneralAgent::new().await?;

    println!("✅ Agent initialized successfully");
    println!("🔑 Agent Public Key: {}", agent.keys.public_key());

    // Handle different modes
    match matches.subcommand() {
        ("demo", _) => {
            agent.run_demo().await?;
        }
        ("task", Some(task_matches)) => {
            let description = task_matches.value_of("description").unwrap();
            agent.process_task_request(description).await?;
        }
        ("interactive", _) | _ => {
            agent.run_cli().await?;
        }
    }

    Ok(())
}

// Additional utility modules

pub mod utils {
    use super::*;

    pub fn format_sats(sats: u64) -> String {
        if sats >= 100_000_000 {
            format!("{:.2} BTC", sats as f64 / 100_000_000.0)
        } else if sats >= 1000 {
            format!("{:.1}k sats", sats as f64 / 1000.0)
        } else {
            format!("{} sats", sats)
        }
    }

    pub fn format_duration(minutes: u32) -> String {
        if minutes >= 60 {
            let hours = minutes / 60;
            let mins = minutes % 60;
            if mins == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}m", hours, mins)
            }
        } else {
            format!("{}m", minutes)
        }
    }

    pub fn truncate_pubkey(pubkey: &str, len: usize) -> String {
        if pubkey.len() <= len {
            pubkey.to_string()
        } else {
            format!("{}...", &pubkey[..len])
        }
    }
}

// Integration tests
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_agent_initialization() {
        let agent = GeneralAgent::new().await;
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_task_decomposition() {
        let agent = GeneralAgent::new().await.unwrap();
        let result = agent.task_decomposer.decompose("Analyze protein structure");

        assert!(result.is_ok());
        let task_tree = result.unwrap();
        assert!(task_tree.root.subtasks.len() > 0);
        assert!(task_tree.total_estimated_cost > 0);
    }

    #[tokio::test]
    async fn test_marketplace_simulation() {
        let mut agent = GeneralAgent::new().await.unwrap();

        let task_request = TaskRequest {
            task_id: "test_task".to_string(),
            requester_pubkey: agent.keys.public_key().to_string(),
            description: "Test task".to_string(),
            required_skills: vec!["testing".to_string()],
            max_budget_sats: 1000,
            deadline_minutes: 60,
            quality_requirements: QualityRequirements {
                min_reputation: 3.0,
                min_success_rate: 0.8,
                requires_validation: false,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        };

        let bids = agent.marketplace_client.simulate_bids(&task_request).await;
        assert!(bids.len() > 0);
    }

    #[tokio::test]
    async fn test_payment_escrow() {
        let mut agent = GeneralAgent::new().await.unwrap();

        let conditions = EscrowConditions {
            requires_verification: true,
            min_quality_rating: 4.0,
            deadline_unix: chrono::Utc::now().timestamp() as u64 + 3600,
            penalty_percentage: 20,
            auto_release_hours: 24,
        };

        let result = agent
            .payment_system
            .create_escrow(
                "test_task",
                "requester_pubkey",
                "specialist_pubkey",
                1000,
                conditions,
            )
            .await;

        assert!(result.is_ok());
        let escrow = result.unwrap();
        assert_eq!(escrow.amount_sats, 1000);
    }
}

// Example configuration file support
pub mod config {
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AgentConfig {
        pub relay_urls: Vec<String>,
        pub mint_urls: Vec<String>,
        pub default_budget_sats: u64,
        pub default_deadline_minutes: u32,
        pub min_reputation_threshold: f64,
        pub auto_approve_low_risk_tasks: bool,
        pub privacy_level: String,
        pub compliance_mode: bool,
    }

    impl Default for AgentConfig {
        fn default() -> Self {
            Self {
                relay_urls: vec!["ws://localhost:8080".to_string()],
                mint_urls: vec!["http://localhost:3338".to_string()],
                default_budget_sats: 5000,
                default_deadline_minutes: 120,
                min_reputation_threshold: 4.0,
                auto_approve_low_risk_tasks: false,
                privacy_level: "participants".to_string(),
                compliance_mode: false,
            }
        }
    }

    impl AgentConfig {
        pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let content = fs::read_to_string(path)?;
            let config: AgentConfig = toml::from_str(&content)?;
            Ok(config)
        }

        pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let content = toml::to_string_pretty(self)?;
            fs::write(path, content)?;
            Ok(())
        }
    }
}

// Example Cargo.toml dependencies needed:
/*
[dependencies]
tokio = { version = "1.0", features = ["full"] }
clap = "2.34"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
nostr = "0.35"
nostr-sdk = "0.33"
cashu = "0.2"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
aes-gcm = "0.10"
rand = "0.8"
base64 = "0.22"
hex = "0.4"
toml = "0.8"

[dev-dependencies]
tokio-test = "0.4"
*/
