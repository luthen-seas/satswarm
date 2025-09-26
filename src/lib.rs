// src/lib.rs - Main Satswarm Protocol Integration
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use nostr::Keys;

// Import all modules
pub mod decomposition;
pub mod marketplace;
pub mod payments;
pub mod reputation;
pub mod privacy;
pub mod observability;
pub mod relay;

use decomposition::{TaskDecomposer, TaskTree, TaskStatus};
use marketplace::{MarketplaceClient, TaskRequest, TaskBid, QualityRequirements};
use payments::{SatswarmPaymentSystem, PaymentEscrow, EscrowConditions, EscrowStatus};
use reputation::{ReputationManager, TaskRating, ReputationScore};
use privacy::{PrivacyManager, ComplianceManager, ContentType, PrivacyLevel, AccessControl};
use observability::{ObservabilityDashboard, TaskEventType, AlertSeverity, AlertCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatswarmConfig {
    pub relay_urls: Vec<String>,
    pub mint_urls: Vec<String>,
    pub agent_keys: Option<String>, // hex-encoded private key
    pub default_privacy_level: PrivacyLevel,
    pub compliance_enabled: bool,
    pub min_reputation_threshold: f64,
    pub auto_verification_threshold: f64,
    pub deflationary_pool_initial: u64,
}

impl Default for SatswarmConfig {
    fn default() -> Self {
        Self {
            relay_urls: vec!["ws://localhost:8080".to_string()],
            mint_urls: vec!["http://localhost:3338".to_string(), "http://localhost:3339".to_string()],
            agent_keys: None,
            default_privacy_level: PrivacyLevel::Participants,
            compliance_enabled: true,
            min_reputation_threshold: 4.0,
            auto_verification_threshold: 4.5,
            deflationary_pool_initial: 100000, // 100k sats
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task_id: String,
    pub requester_pubkey: String,
    pub specialist_pubkey: Option<String>,
    pub task_tree: TaskTree,
    pub selected_bid: Option<TaskBid>,
    pub escrow: Option<PaymentEscrow>,
    pub status: TaskExecutionStatus,
    pub verification_result: Option<VerificationResult>,
    pub rating: Option<TaskRating>,
    pub created_at: u64,
    pub updated_at: u64,
    pub execution_log: Vec<ExecutionLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskExecutionStatus {
    Decomposed,
    Broadcast,
    BiddingOpen,
    BiddingClosed,
    Assigned,
    InProgress,
    Submitted,
    UnderVerification,
    Verified,
    PaymentReleased,
    Completed,
    Failed,
    Disputed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verifier_id: String,
    pub verification_method: VerificationMethod,
    pub quality_score: f64,
    pub fraud_detected: bool,
    pub verification_proof: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    MockVerification,
    BitVMProof,
    ZKProof,
    PeerReview,
    AutomatedCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: u64,
    pub event_type: TaskEventType,
    pub participant: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct SatswarmProtocol {
    pub config: SatswarmConfig,
    pub keys: Keys,
    pub task_decomposer: TaskDecomposer,
    pub marketplace_client: Arc<RwLock<MarketplaceClient>>,
    pub payment_system: Arc<RwLock<SatswarmPaymentSystem>>,
    pub reputation_manager: Arc<RwLock<ReputationManager>>,
    pub privacy_manager: Arc<RwLock<PrivacyManager>>,
    pub compliance_manager: Arc<RwLock<ComplianceManager>>,
    pub dashboard: Arc<RwLock<ObservabilityDashboard>>,
    pub active_tasks: Arc<RwLock<HashMap<String, TaskExecution>>>,
    pub relay_manager: Arc<RwLock<relay::RelayManager>>,
}

impl SatswarmProtocol {
    pub async fn new(config: SatswarmConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize keys
        let keys = if let Some(key_hex) = &config.agent_keys {
            Keys::from_sk_str(key_hex)?
        } else {
            Keys::generate()
        };

        // Initialize relay manager
        let relay_manager = Arc::new(RwLock::new(
            relay::RelayManager::new(config.relay_urls.clone()).await?
        ));

        // Initialize marketplace client
        let marketplace_client = Arc::new(RwLock::new(
            MarketplaceClient::new(&config.relay_urls[0]).await?
        ));

        // Initialize payment system
        let mut payment_system = SatswarmPaymentSystem::new();
        payment_system.deflationary_pool.total_pool_sats = config.deflationary_pool_initial;

        // Initialize dashboard
        let dashboard = Arc::new(RwLock::new(ObservabilityDashboard::new()));

        Ok(Self {
            config,
            keys,
            task_decomposer: TaskDecomposer::new(),
            marketplace_client,
            payment_system: Arc::new(RwLock::new(payment_system)),
            reputation_manager: Arc::new(RwLock::new(ReputationManager::new())),
            privacy_manager: Arc::new(RwLock::new(PrivacyManager::new())),
            compliance_manager: Arc::new(RwLock::new(ComplianceManager::new())),
            dashboard,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            relay_manager,
        })
    }

    // Complete end-to-end task processing
    pub async fn process_task_request(
        &self,
        request_description: &str,
        max_budget_sats: u64,
        deadline_minutes: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        
        println!("🚀 Starting end-to-end task processing...");
        let start_time = std::time::Instant::now();

        // Step 1: Task Decomposition
        let task_tree = self.decompose_task(request_description).await?;
        let task_id = task_tree.root.id.clone();

        println!("✅ Task decomposed: {} subtasks, {} sats estimated", 
                 task_tree.root.subtasks.len(), task_tree.total_estimated_cost);

        // Step 2: Create Task Execution Record
        let task_execution = TaskExecution {
            task_id: task_id.clone(),
            requester_pubkey: self.keys.public_key().to_string(),
            specialist_pubkey: None,
            task_tree,
            selected_bid: None,
            escrow: None,
            status: TaskExecutionStatus::Decomposed,
            verification_result: None,
            rating: None,
            created_at: Utc::now().timestamp() as u64,
            updated_at: Utc::now().timestamp() as u64,
            execution_log: vec![
                ExecutionLogEntry {
                    timestamp: Utc::now().timestamp() as u64,
                    event_type: TaskEventType::TaskCreated,
                    participant: self.keys.public_key().to_string(),
                    message: "Task created and decomposed".to_string(),
                    metadata: HashMap::new(),
                }
            ],
        };

        // Store task execution
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(task_id.clone(), task_execution.clone());
        }

        // Step 3: Broadcast to Marketplace
        let task_request = self.create_task_request(&task_execution, max_budget_sats, deadline_minutes).await?;
        self.broadcast_task(&task_request).await?;

        // Update status
        self.update_task_status(&task_id, TaskExecutionStatus::Broadcast).await?;

        // Step 4: Collect Bids
        let bids = self.collect_bids(&task_id, 10).await?; // 10 second timeout for demo
        
        if bids.is_empty() {
            self.update_task_status(&task_id, TaskExecutionStatus::Failed).await?;
            return Err("No bids received".into());
        }

        println!("📥 Received {} bids", bids.len());
        self.update_task_status(&task_id, TaskExecutionStatus::BiddingClosed).await?;

        // Step 5: Select Winning Bid
        let winning_bid = self.select_winning_bid(bids).await?;
        
        // Update task with selected bid
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.specialist_pubkey = Some(winning_bid.specialist_pubkey.clone());
                task.selected_bid = Some(winning_bid.clone());
            }
        }

        println!("🏆 Selected specialist: {}", &winning_bid.specialist_pubkey[..16]);
        self.update_task_status(&task_id, TaskExecutionStatus::Assigned).await?;

        // Step 6: Create Payment Escrow
        let escrow = self.create_payment_escrow(&task_id, &winning_bid).await?;
        
        // Update task with escrow
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.escrow = Some(escrow.clone());
            }
        }

        println!("💰 Escrow created: {} sats locked", escrow.amount_sats);
        self.update_task_status(&task_id, TaskExecutionStatus::InProgress).await?;

        // Step 7: Simulate Task Execution
        self.simulate_task_execution(&task_id).await?;
        self.update_task_status(&task_id, TaskExecutionStatus::Submitted).await?;

        // Step 8: Verification
        let verification_result = self.verify_task_output(&task_id).await?;
        
        // Update task with verification
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.verification_result = Some(verification_result.clone());
            }
        }

        println!("✅ Verification completed: Quality {:.1}/5.0", verification_result.quality_score);
        self.update_task_status(&task_id, TaskExecutionStatus::Verified).await?;

        // Step 9: Release Payment
        self.release_payment(&task_id, &verification_result).await?;
        self.update_task_status(&task_id, TaskExecutionStatus::PaymentReleased).await?;

        // Step 10: Update Reputation & Economic Loops
        self.update_reputation(&task_id, &verification_result).await?;
        self.process_economic_rewards(&task_id).await?;

        // Final status
        self.update_task_status(&task_id, TaskExecutionStatus::Completed).await?;

        let elapsed = start_time.elapsed();
        println!("🎉 Task completed successfully in {:.2}s", elapsed.as_secs_f64());

        // Update dashboard metrics
        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.record_performance("total_task_time_ms", elapsed.as_millis() as f64);
            dashboard.update_metrics(5, 0, 1, winning_bid.quoted_price_sats, 0);
        }

        Ok(task_id)
    }

    async fn decompose_task(&self, request: &str) -> Result<TaskTree, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let task_tree = self.task_decomposer.decompose(request)?;
        
        let elapsed = start_time.elapsed().as_millis() as f64;
        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.record_performance("decomposition_time_ms", elapsed);
        }

        Ok(task_tree)
    }

    async fn create_task_request(
        &self,
        task_execution: &TaskExecution,
        max_budget_sats: u64,
        deadline_minutes: u32,
    ) -> Result<TaskRequest, Box<dyn std::error::Error>> {
        
        let required_skills: Vec<String> = task_execution.task_tree.root.subtasks
            .iter()
            .flat_map(|subtask| subtask.required_skills.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(TaskRequest {
            task_id: task_execution.task_id.clone(),
            requester_pubkey: task_execution.requester_pubkey.clone(),
            description: task_execution.task_tree.root.description.clone(),
            required_skills,
            max_budget_sats,
            deadline_minutes,
            quality_requirements: QualityRequirements {
                min_reputation: self.config.min_reputation_threshold,
                min_success_rate: 0.9,
                requires_verification: true,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        })
    }

    async fn broadcast_task(&self, task_request: &TaskRequest) -> Result<(), Box<dyn std::error::Error>> {
        let marketplace = self.marketplace_client.write().await;
        marketplace.broadcast_task_request(task_request, &self.keys).await?;

        // Log to dashboard
        let mut dashboard = self.dashboard.write().await;
        dashboard.record_task_event(
            &task_request.task_id,
            TaskEventType::TaskBroadcast,
            &task_request.requester_pubkey,
            HashMap::new(),
        );

        Ok(())
    }

    async fn collect_bids(&self, task_id: &str, timeout_seconds: u64) -> Result<Vec<TaskBid>, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Get task request for bid simulation
        let task_request = {
            let tasks = self.active_tasks.read().await;
            let task = tasks.get(task_id).ok_or("Task not found")?;
            
            TaskRequest {
                task_id: task.task_id.clone(),
                requester_pubkey: task.requester_pubkey.clone(),
                description: task.task_tree.root.description.clone(),
                required_skills: task.task_tree.root.subtasks
                    .iter()
                    .flat_map(|st| st.required_skills.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
                max_budget_sats: task.task_tree.total_estimated_cost,
                deadline_minutes: 120,
                quality_requirements: QualityRequirements {
                    min_reputation: self.config.min_reputation_threshold,
                    min_success_rate: 0.9,
                    requires_verification: true,
                    preferred_specialists: Vec::new(),
                    blacklisted_specialists: Vec::new(),
                },
            }
        };

        // For MVP, use simulation
        let marketplace = self.marketplace_client.read().await;
        let bids = marketplace.simulate_bids(&task_request).await;

        // Record bid events
        let mut dashboard = self.dashboard.write().await;
        for bid in &bids {
            dashboard.record_task_event(
                task_id,
                TaskEventType::BidReceived,
                &bid.specialist_pubkey,
                HashMap::new(),
            );
        }

        let elapsed = start_time.elapsed().as_millis() as f64;
        dashboard.record_performance("matching_time_ms", elapsed);

        Ok(bids)
    }

    async fn select_winning_bid(&self, bids: Vec<TaskBid>) -> Result<TaskBid, Box<dyn std::error::Error>> {
        let marketplace = self.marketplace_client.read().await;
        
        marketplace.select_winning_bid(bids)
            .await
            .ok_or_else(|| "No suitable bid found".into())
    }

    async fn create_payment_escrow(&self, task_id: &str, winning_bid: &TaskBid) -> Result<PaymentEscrow, Box<dyn std::error::Error>> {
        let conditions = EscrowConditions {
            requires_verification: true,
            min_quality_rating: self.config.min_reputation_threshold,
            deadline_unix: Utc::now().timestamp() as u64 + 7200, // 2 hours
            penalty_percentage: 20,
            auto_release_hours: 24,
        };

        let mut payment_system = self.payment_system.write().await;
        let escrow = payment_system.create_escrow(
            task_id,
            &self.keys.public_key().to_string(),
            &winning_bid.specialist_pubkey,
            winning_bid.quoted_price_sats,
            conditions,
        ).await?;

        Ok(escrow)
    }

    async fn simulate_task_execution(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔬 Specialist working on task...");
        
        // Simulate work time
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        
        // Log work submission
        let mut dashboard = self.dashboard.write().await;
        dashboard.record_task_event(
            task_id,
            TaskEventType::WorkSubmitted,
            "specialist",
            HashMap::new(),
        );

        println!("📋 Work completed and submitted");
        Ok(())
    }

    async fn verify_task_output(&self, task_id: &str) -> Result<VerificationResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        println!("🔍 Verifying task output...");
        
        // Log verification start
        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.record_task_event(
                task_id,
                TaskEventType::VerificationStarted,
                "verifier_node",
                HashMap::new(),
            );
        }

        // Simulate verification time
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // Mock verification result (high quality for demo)
        let quality_score = 4.6 + (rand::random::<f64>() * 0.4); // 4.6-5.0 range
        
        let verification_result = VerificationResult {
            verifier_id: "mock_verifier".to_string(),
            verification_method: VerificationMethod::MockVerification,
            quality_score,
            fraud_detected: false,
            verification_proof: format!("mock_proof_{}", Uuid::new_v4()),
            timestamp: Utc::now().timestamp() as u64,
        };

        // Log verification completion
        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.record_task_event(
                task_id,
                TaskEventType::VerificationCompleted,
                "verifier_node",
                HashMap::new(),
            );

            let elapsed = start_time.elapsed().as_millis() as f64;
            dashboard.record_performance("verification_time_ms", elapsed);
        }

        Ok(verification_result)
    }

    async fn release_payment(&self, task_id: &str, verification: &VerificationResult) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // Get escrow ID
        let escrow_id = {
            let tasks = self.active_tasks.read().await;
            let task = tasks.get(task_id).ok_or("Task not found")?;
            task.escrow.as_ref().ok_or("No escrow found")?.escrow_id.clone()
        };

        // Release payment
        let mut payment_system = self.payment_system.write().await;
        payment_system.release_escrow(&escrow_id, verification.quality_score).await?;

        // Log payment release
        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.record_task_event(
                task_id,
                TaskEventType::PaymentReleased,
                "payment_system",
                HashMap::new(),
            );

            let elapsed = start_time.elapsed().as_millis() as f64;
            dashboard.record_performance("payment_processing_time_ms", elapsed);
        }

        Ok(())
    }

    async fn update_reputation(&self, task_id: &str, verification: &VerificationResult) -> Result<(), Box<dyn std::error::Error>> {
        let (specialist_pubkey, task_description) = {
            let tasks = self.active_tasks.read().await;
            let task = tasks.get(task_id).ok_or("Task not found")?;
            (
                task.specialist_pubkey.as_ref().unwrap().clone(),
                task.task_tree.root.description.clone(),
            )
        };

        let rating = TaskRating {
            rating_id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            specialist_pubkey: specialist_pubkey.clone(),
            requester_pubkey: self.keys.public_key().to_string(),
            quality_score: verification.quality_score,
            timeliness_score: 4.8, // Mock high timeliness
            communication_score: 4.5,
            would_work_again: verification.quality_score >= 4.0,
            comment: format!("Task completed: {}", task_description),
            verified_completion: !verification.fraud_detected,
            created_at: Utc::now().timestamp() as u64,
            signature: "mock_signature".to_string(),
        };

        let mut reputation_manager = self.reputation_manager.write().await;
        reputation_manager.submit_rating(rating, &self.keys)?;

        println!("⭐ Reputation updated for specialist");
        Ok(())
    }

    async fn process_economic_rewards(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut payment_system = self.payment_system.write().await;
        let distributions = payment_system.distribute_deflationary_rewards().await?;

        if !distributions.is_empty() {
            println!("🎁 Economic rewards distributed:");
            for (pubkey, amount) in distributions {
                println!("  {}: {} sats", &pubkey[..12], amount);
            }
        }

        Ok(())
    }

    async fn update_task_status(&self, task_id: &str, status: TaskExecutionStatus) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = status.clone();
            task.updated_at = Utc::now().timestamp() as u64;
            
            // Add log entry
            task.execution_log.push(ExecutionLogEntry {
                timestamp: Utc::now().timestamp() as u64,
                event_type: match status {
                    TaskExecutionStatus::Decomposed => TaskEventType::TaskCreated,
                    TaskExecutionStatus::Broadcast => TaskEventType::TaskBroadcast,
                    TaskExecutionStatus::Assigned => TaskEventType::TaskAssigned,
                    TaskExecutionStatus::InProgress => TaskEventType::WorkStarted,
                    TaskExecutionStatus::Submitted => TaskEventType::WorkSubmitted,
                    TaskExecutionStatus::Verified => TaskEventType::VerificationCompleted,
                    TaskExecutionStatus::PaymentReleased => TaskEventType::PaymentReleased,
                    TaskExecutionStatus::Completed => TaskEventType::TaskCompleted,
                    TaskExecutionStatus::Failed => TaskEventType::TaskFailed,
                    _ => TaskEventType::TaskCreated,
                },
                participant: self.keys.public_key().to_string(),
                message: format!("Status updated to: {:?}", status),
                metadata: HashMap::new(),
            });
        }

        Ok(())
    }

    // Public API methods
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskExecution> {
        let tasks = self.active_tasks.read().await;
        tasks.get(task_id).cloned()
    }

    pub async fn list_active_tasks(&self) -> Vec<TaskExecution> {
        let tasks = self.active_tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn get_dashboard_summary(&self) -> Result<String, Box<dyn std::error::Error>> {
        let dashboard = self.dashboard.read().await;
        dashboard.export_metrics_json()
    }

    pub async fn print_system_status(&self) {
        let dashboard = self.dashboard.read().await;
        dashboard.print_dashboard();
        
        let payment_system = self.payment_system.read().await;
        payment_system.print_economic_status();
    }

    pub fn get_agent_pubkey(&self) -> String {
        self.keys.public_key().to_string()
    }
}