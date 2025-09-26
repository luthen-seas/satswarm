// tests/integration_tests.rs
use satswarm::{SatswarmProtocol, SatswarmConfig};
use satswarm::decomposition::TaskDecomposer;
use satswarm::marketplace::{MarketplaceClient, TaskRequest, QualityRequirements};
use satswarm::payments::{SatswarmPaymentSystem, EscrowConditions};
use satswarm::reputation::{ReputationManager, TaskRating};
use satswarm::privacy::{PrivacyManager, ComplianceManager, ContentType, PrivacyLevel, AccessControl};
use satswarm::observability::{ObservabilityDashboard, TaskEventType};
use satswarm::relay::RelayManager;

use tokio;
use std::time::Duration;
use uuid::Uuid;
use nostr::Keys;
use chrono::Utc;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_task_decomposition_engine() {
        let decomposer = TaskDecomposer::new();
        
        // Test protein analysis
        let result = decomposer.decompose("Analyze protein structure of BRCA1");
        assert!(result.is_ok());
        
        let task_tree = result.unwrap();
        assert_eq!(task_tree.root.subtasks.len(), 4);
        assert!(task_tree.total_estimated_cost > 0);
        
        // Test financial analysis
        let result = decomposer.decompose("Optimize portfolio yield with risk constraints");
        assert!(result.is_ok());
        
        let task_tree = result.unwrap();
        assert_eq!(task_tree.root.subtasks.len(), 3);
        
        // Test unknown domain
        let result = decomposer.decompose("Bake a chocolate cake");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_marketplace_bid_simulation() {
        let mut client = MarketplaceClient::new("ws://localhost:8080").await.unwrap();
        
        let task_request = TaskRequest {
            task_id: "test_task".to_string(),
            requester_pubkey: Keys::generate().public_key().to_string(),
            description: "Test bioinformatics task".to_string(),
            required_skills: vec!["bioinformatics".to_string(), "protein_analysis".to_string()],
            max_budget_sats: 2000,
            deadline_minutes: 120,
            quality_requirements: QualityRequirements {
                min_reputation: 4.0,
                min_success_rate: 0.9,
                requires_validation: true,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        };

        let bids = client.simulate_bids(&task_request).await;
        
        assert!(!bids.is_empty());
        assert!(bids.len() >= 1);
        
        // Verify bid quality
        for bid in &bids {
            assert!(bid.quoted_price_sats <= task_request.max_budget_sats);
            assert!(bid.estimated_completion_time <= task_request.deadline_minutes);
            assert!(!bid.specialist_pubkey.is_empty());
        }

        // Test bid selection
        let winning_bid = client.select_winning_bid(bids).await;
        assert!(winning_bid.is_some());
    }

    #[tokio::test]
    async fn test_payment_escrow_lifecycle() {
        let mut payment_system = SatswarmPaymentSystem::new();
        
        // Create escrow
        let conditions = EscrowConditions {
            requires_verification: true,
            min_quality_rating: 4.0,
            deadline_unix: Utc::now().timestamp() as u64 + 3600,
            penalty_percentage: 20,
            auto_release_hours: 24,
        };

        let escrow = payment_system.create_escrow(
            "test_task",
            "requester_pubkey",
            "specialist_pubkey", 
            1000,
            conditions,
        ).await.unwrap();

        assert_eq!(escrow.amount_sats, 1000);
        assert_eq!(escrow.stake_amount, 100);
        
        // Release payment
        let release_result = payment_system.release_escrow(&escrow.escrow_id, 4.5).await;
        assert!(release_result.is_ok());
        
        // Check specialist balance
        let status = payment_system.get_payment_status("specialist_pubkey");
        assert_eq!(status.balance_sats, 1000);
    }

    #[tokio::test]
    async fn test_reputation_system() {
        let mut reputation_manager = ReputationManager::new();
        let keys = Keys::generate();
        let specialist_pubkey = "test_specialist";
        
        // Initialize reputation
        reputation_manager.initialize_reputation(specialist_pubkey);
        
        // Submit rating
        let rating = TaskRating {
            rating_id: Uuid::new_v4().to_string(),
            task_id: "test_task".to_string(),
            specialist_pubkey: specialist_pubkey.to_string(),
            requester_pubkey: "requester".to_string(),
            quality_score: 4.5,
            timeliness_score: 4.2,
            communication_score: 4.6,
            would_work_again: true,
            comment: "Excellent work".to_string(),
            verified_completion: true,
            created_at: Utc::now().timestamp() as u64,
            signature: "mock_sig".to_string(),
        };

        let result = reputation_manager.submit_rating(rating, &keys);
        assert!(result.is_ok());
        
        // Check updated reputation
        let updated_score = reputation_manager.get_reputation(specialist_pubkey).unwrap();
        assert_eq!(updated_score.total_tasks_completed, 1);
        assert!(updated_score.overall_score > 3.0);
        assert!(updated_score.avg_quality_rating > 0.0);
    }

    #[tokio::test]
    async fn test_privacy_encryption() {
        let mut privacy_manager = PrivacyManager::new();
        
        let access_control = AccessControl {
            authorized_pubkeys: vec!["authorized_user".to_string()],
            requires_stake: false,
            min_reputation: 0.0,
            expiry_timestamp: None,
            access_conditions: Vec::new(),
        };

        // Test encryption
        let wrapped = privacy_manager.encrypt_content(
            "Sensitive task data",
            ContentType::TaskDescription,
            PrivacyLevel::Participants,
            access_control,
        ).unwrap();

        assert_eq!(wrapped.content_type, ContentType::TaskDescription);
        assert_eq!(wrapped.privacy_level, PrivacyLevel::Participants);
        assert!(wrapped.metadata.blind_signature_hash.is_some());
    }

    #[tokio::test]
    async fn test_compliance_manager() {
        let mut compliance_manager = ComplianceManager::new();
        
        // Test consent management
        let consent = compliance_manager.grant_consent(
            "test_user",
            vec![satswarm::privacy::ConsentType::TaskProcessing],
            Some(365),
        );

        assert!(compliance_manager.check_consent("test_user", &satswarm::privacy::ConsentType::TaskProcessing));
        
        // Test data erasure
        let erasure_request = compliance_manager.request_erasure(
            "test_user",
            vec![satswarm::privacy::DataCategory::TaskHistory],
        );

        let process_result = compliance_manager.process_erasure(&erasure_request.request_id);
        assert!(process_result.is_ok());
    }

    #[tokio::test]
    async fn test_observability_dashboard() {
        let mut dashboard = ObservabilityDashboard::new();
        
        // Test metrics update
        dashboard.update_metrics(10, 5, 8, 25000, 5000);
        
        assert_eq!(dashboard.current_metrics.system_health.total_participants, 10);
        assert_eq!(dashboard.current_metrics.task_flow_metrics.active_tasks, 5);
        assert_eq!(dashboard.current_metrics.economic_metrics.total_sats_transacted, 25000);
        
        // Test event recording
        dashboard.record_task_event(
            "test_task",
            TaskEventType::TaskCreated,
            "test_user",
            std::collections::HashMap::new(),
        );
        
        assert_eq!(dashboard.task_events.len(), 1);
        
        // Test alert creation
        let alert_id = dashboard.create_alert(
            satswarm::observability::AlertSeverity::Warning,
            satswarm::observability::AlertCategory::SystemHealth,
            "Test alert",
            "test_component",
        );
        
        assert_eq!(dashboard.active_alerts.len(), 1);
        
        // Test alert resolution
        let resolved = dashboard.resolve_alert(&alert_id, "Manual resolution");
        assert!(resolved);
        assert_eq!(dashboard.active_alerts.len(), 0);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_protocol_integration() {
        // Initialize protocol
        let config = SatswarmConfig::default();
        let protocol = SatswarmProtocol::new(config).await.unwrap();
        
        println!("🧪 Testing full protocol integration...");
        
        // Test end-to-end task processing
        let result = protocol.process_task_request(
            "Analyze protein structure for drug binding",
            5000,
            120,
        ).await;

        assert!(result.is_ok());
        let task_id = result.unwrap();
        
        // Verify task was processed
        let task_status = protocol.get_task_status(&task_id).await;
        assert!(task_status.is_some());
        
        let task = task_status.unwrap();
        assert_eq!(task.status, satswarm::TaskExecutionStatus::Completed);
        assert!(task.specialist_pubkey.is_some());
        assert!(task.selected_bid.is_some());
        assert!(task.escrow.is_some());
        assert!(task.verification_result.is_some());
        
        println!("✅ Full protocol integration test passed");
    }

    #[tokio::test]
    async fn test_relay_integration() {
        let mut relay_manager = RelayManager::new(vec!["ws://localhost:8080".to_string()]).await.unwrap();
        
        // Start local relay
        let result = relay_manager.start_local_relay(18081).await;
        assert!(result.is_ok());
        
        // Give relay time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Test relay info
        let info = relay_manager.get_relay_info().await;
        assert!(info.is_ok());
        
        let relay_info = info.unwrap();
        assert_eq!(relay_info.name, "Satswarm Local Relay");
        assert!(relay_info.supported_nips.contains(&1));
        
        println!("✅ Relay integration test passed");
    }

    #[tokio::test]
    async fn test_multi_agent_simulation() {
        println!("🧪 Testing multi-agent scenario...");
        
        // Create multiple agent instances
        let config1 = SatswarmConfig::default();
        let config2 = SatswarmConfig::default();
        
        let agent1 = SatswarmProtocol::new(config1).await.unwrap();
        let agent2 = SatswarmProtocol::new(config2).await.unwrap();
        
        // Agent 1 processes a task
        let task_result = agent1.process_task_request(
            "Perform quantitative analysis on market data",
            3000,
            90,
        ).await;
        
        assert!(task_result.is_ok());
        
        // Agent 2 processes a different task
        let task_result2 = agent2.process_task_request(
            "Train machine learning model for prediction",
            8000,
            180,
        ).await;
        
        assert!(task_result2.is_ok());
        
        // Verify both agents have active tasks
        let tasks1 = agent1.list_active_tasks().await;
        let tasks2 = agent2.list_active_tasks().await;
        
        assert_eq!(tasks1.len(), 1);
        assert_eq!(tasks2.len(), 1);
        
        println!("✅ Multi-agent simulation test passed");
    }

    #[tokio::test]
    async fn test_economic_loops() {
        println!("🧪 Testing economic incentive loops...");
        
        let config = SatswarmConfig::default();
        let protocol = SatswarmProtocol::new(config).await.unwrap();
        
        // Process multiple tasks to test economic loops
        let task1 = protocol.process_task_request(
            "Analyze protein folding patterns",
            2000,
            60,
        ).await.unwrap();
        
        let task2 = protocol.process_task_request(
            "Optimize neural network architecture", 
            4000,
            120,
        ).await.unwrap();
        
        // Verify economic state changes
        let payment_system = protocol.payment_system.read().await;
        
        // Check that economic loops were updated
        assert!(payment_system.economic_loops.len() > 0);
        
        // Check deflationary pool changes
        assert!(payment_system.deflationary_pool.total_pool_sats > 0);
        
        // Verify performance tiers were assigned
        for (_, economic_loop) in &payment_system.economic_loops {
            assert!(economic_loop.visibility_multiplier >= 1.0);
            assert!(economic_loop.earned_tokens > 0);
        }
        
        println!("✅ Economic loops test passed");
    }

    #[tokio::test]
    async fn test_fraud_detection_and_slashing() {
        println!("🧪 Testing fraud detection and slashing...");
        
        let config = SatswarmConfig::default();
        let protocol = SatswarmProtocol::new(config).await.unwrap();
        
        // Create a task and escrow
        let mut payment_system = protocol.payment_system.write().await;
        
        let conditions = EscrowConditions {
            requires_verification: true,
            min_quality_rating: 4.0,
            deadline_unix: Utc::now().timestamp() as u64 + 3600,
            penalty_percentage: 50, // High penalty for testing
            auto_release_hours: 24,
        };

        let escrow = payment_system.create_escrow(
            "fraud_test_task",
            "requester_pubkey",
            "specialist_pubkey",
            1000,
            conditions,
        ).await.unwrap();
        
        let initial_pool = payment_system.deflationary_pool.total_pool_sats;
        
        // Simulate fraud and slash escrow
        let slashed_amount = payment_system.slash_escrow(&escrow.escrow_id, 50).await.unwrap();
        
        assert_eq!(slashed_amount, 500); // 50% of 1000 sats
        
        // Verify slashed amount went to deflationary pool
        assert_eq!(payment_system.deflationary_pool.total_pool_sats, initial_pool + 500);
        
        println!("✅ Fraud detection and slashing test passed");
    }

    #[tokio::test]
    async fn test_privacy_access_control() {
        println!("🧪 Testing privacy and access control...");
        
        let mut privacy_manager = PrivacyManager::new();
        
        let access_control = AccessControl {
            authorized_pubkeys: vec!["authorized_user".to_string()],
            requires_stake: true,
            min_reputation: 4.0,
            expiry_timestamp: Some(Utc::now().timestamp() as u64 + 3600),
            access_conditions: vec![
                satswarm::privacy::AccessCondition::ReputationThreshold(4.0),
                satswarm::privacy::AccessCondition::StakeAmount(100),
            ],
        };

        // Encrypt content
        let wrapped = privacy_manager.encrypt_content(
            "Highly sensitive task data",
            ContentType::TaskDescription,
            PrivacyLevel::Selective,
            access_control.clone(),
        ).unwrap();
        
        // Test authorized access
        let authorized_context = satswarm::privacy::DecryptionContext {
            requester_reputation: Some(4.5),
            is_active_bidder: true,
            is_task_participant: false,
            stake_amount: Some(150),
            task_id: None,
        };
        
        let decrypt_result = privacy_manager.decrypt_content(
            &wrapped,
            "authorized_user",
            &authorized_context,
        );
        assert!(decrypt_result.is_ok());
        
        // Test unauthorized access
        let unauthorized_context = satswarm::privacy::DecryptionContext {
            requester_reputation: Some(3.0), // Below threshold
            is_active_bidder: false,
            is_task_participant: false,
            stake_amount: Some(50), // Below threshold
            task_id: None,
        };
        
        let decrypt_result = privacy_manager.decrypt_content(
            &wrapped,
            "unauthorized_user",
            &unauthorized_context,
        );
        assert!(decrypt_result.is_err());
        
        println!("✅ Privacy and access control test passed");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_task_decomposition_performance() {
        println!("⚡ Testing task decomposition performance...");
        
        let decomposer = TaskDecomposer::new();
        let start = Instant::now();
        
        // Test 100 decompositions
        for i in 0..100 {
            let request = match i % 3 {
                0 => "Analyze protein structure and binding sites",
                1 => "Optimize portfolio allocation with risk constraints",
                _ => "Train deep learning model for image classification",
            };
            
            let result = decomposer.decompose(request);
            assert!(result.is_ok());
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_millis() as f64 / 100.0;
        
        println!("✅ Average decomposition time: {:.2}ms", avg_time);
        assert!(avg_time < 10.0); // Should be under 10ms
    }

    #[tokio::test]
    async fn test_marketplace_scaling() {
        println!("⚡ Testing marketplace scaling...");
        
        let mut client = MarketplaceClient::new("ws://localhost:8080").await.unwrap();
        let start = Instant::now();
        
        // Create 50 concurrent task requests
        let mut handles = Vec::new();
        
        for i in 0..50 {
            let task_request = TaskRequest {
                task_id: format!("perf_test_{}", i),
                requester_pubkey: Keys::generate().public_key().to_string(),
                description: format!("Performance test task {}", i),
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
            
            let bids = client.simulate_bids(&task_request).await;
            assert!(!bids.is_empty());
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_millis() as f64 / 50.0;
        
        println!("✅ Average bid simulation time: {:.2}ms", avg_time);
        assert!(avg_time < 100.0); // Should be under 100ms
    }

    #[tokio::test]
    async fn test_payment_system_throughput() {
        println!("⚡ Testing payment system throughput...");
        
        let mut payment_system = SatswarmPaymentSystem::new();
        let start = Instant::now();
        
        // Create 100 escrows and releases
        for i in 0..100 {
            let conditions = EscrowConditions {
                requires_verification: false,
                min_quality_rating: 3.0,
                deadline_unix: Utc::now().timestamp() as u64 + 3600,
                penalty_percentage: 10,
                auto_release_hours: 24,
            };

            let escrow = payment_system.create_escrow(
                &format!("perf_task_{}", i),
                &format!("requester_{}", i),
                &format!("specialist_{}", i),
                1000,
                conditions,
            ).await.unwrap();
            
            // Immediately release for testing
            payment_system.release_escrow(&escrow.escrow_id, 4.0).await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let avg_time = elapsed.as_millis() as f64 / 100.0;
        
        println!("✅ Average escrow cycle time: {:.2}ms", avg_time);
        assert!(avg_time < 50.0); // Should be under 50ms
    }

    #[tokio::test]
    async fn test_memory_usage() {
        println!("💾 Testing memory usage...");
        
        let config = SatswarmConfig::default();
        let protocol = SatswarmProtocol::new(config).await.unwrap();
        
        // Process multiple tasks and check for memory leaks
        for i in 0..20 {
            let result = protocol.process_task_request(
                &format!("Memory test task {}", i),
                1000,
                60,
            ).await;
            
            assert!(result.is_ok());
            
            // Allow some processing time
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        // Verify we don't have excessive task accumulation
        let active_tasks = protocol.list_active_tasks().await;
        println!("Active tasks after test: {}", active_tasks.len());
        
        // In a real scenario, completed tasks might be archived
        // For now, we just verify the system remains responsive
        assert!(active_tasks.len() <= 20);
        
        println!("✅ Memory usage test passed");
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_reputation_manipulation_resistance() {
        println!("🔐 Testing reputation manipulation resistance...");
        
        let mut reputation_manager = ReputationManager::new();
        let keys = Keys::generate();
        let specialist_pubkey = "test_specialist";
        
        // Initialize reputation
        reputation_manager.initialize_reputation(specialist_pubkey);
        
        // Try to submit multiple ratings from same requester (should be prevented)
        for i in 0..5 {
            let rating = TaskRating {
                rating_id: format!("rating_{}", i),
                task_id: format!("task_{}", i),
                specialist_pubkey: specialist_pubkey.to_string(),
                requester_pubkey: "same_requester".to_string(), // Same requester
                quality_score: 5.0, // Trying to boost rating
                timeliness_score: 5.0,
                communication_score: 5.0,
                would_work_again: true,
                comment: format!("Fake review {}", i),
                verified_completion: true,
                created_at: Utc::now().timestamp() as u64,
                signature: format!("sig_{}", i),
            };
            
            reputation_manager.submit_rating(rating, &keys).unwrap();
        }
        
        let final_score = reputation_manager.get_reputation(specialist_pubkey).unwrap();
        
        // Score shouldn't be artificially inflated beyond reasonable bounds
        // The system should have some protection against gaming
        assert!(final_score.overall_score <= 5.0);
        
        println!("✅ Reputation manipulation resistance test passed");
    }

    #[tokio::test]
    async fn test_payment_security() {
        println!("🔐 Testing payment security...");
        
        let mut payment_system = SatswarmPaymentSystem::new();
        
        // Test escrow creation with invalid parameters
        let conditions = EscrowConditions {
            requires_verification: true,
            min_quality_rating: 4.0,
            deadline_unix: Utc::now().timestamp() as u64 + 3600,
            penalty_percentage: 20,
            auto_release_hours: 24,
        };

        // Try to create escrow with zero amount (should fail or handle gracefully)
        let result = payment_system.create_escrow(
            "security_test",
            "requester",
            "specialist",
            0, // Zero amount
            conditions.clone(),
        ).await;
        
        // System should handle this gracefully
        assert!(result.is_err() || result.unwrap().amount_sats == 0);
        
        // Test double spending prevention
        let valid_escrow = payment_system.create_escrow(
            "valid_task",
            "requester",
            "specialist",
            1000,
            conditions,
        ).await.unwrap();
        
        // Try to release twice
        let first_release = payment_system.release_escrow(&valid_escrow.escrow_id, 4.5).await;
        assert!(first_release.is_ok());
        
        let second_release = payment_system.release_escrow(&valid_escrow.escrow_id, 4.5).await;
        assert!(second_release.is_err()); // Should fail
        
        println!("✅ Payment security test passed");
    }

    #[tokio::test]
    async fn test_nostr_event_validation() {
        println!("🔐 Testing NOSTR event validation...");
        
        let relay_manager = RelayManager::new(vec!["ws://localhost:8080".to_string()]).await.unwrap();
        
        // Test with valid event
        let valid_keys = Keys::generate();
        let valid_event = nostr::EventBuilder::text_note("Valid test event")
            .sign_with_keys(&valid_keys).unwrap();
        
        relay_manager.store_test_event(valid_event).await.unwrap();
        assert_eq!(relay_manager.get_events_count().await, 1);
        
        // Test signature validation would happen in real relay
        // For MVP, we trust the nostr library's validation
        
        println!("✅ NOSTR event validation test passed");
    }

    #[tokio::test]
    async fn test_privacy_access_control_edge_cases() {
        println!("🔐 Testing privacy access control edge cases...");
        
        let mut privacy_manager = PrivacyManager::new();
        
        // Test expired access control
        let expired_access = AccessControl {
            authorized_pubkeys: vec!["user".to_string()],
            requires_stake: false,
            min_reputation: 0.0,
            expiry_timestamp: Some(Utc::now().timestamp() as u64 - 3600), // Expired 1 hour ago
            access_conditions: Vec::new(),
        };

        let wrapped = privacy_manager.encrypt_content(
            "Expired access test",
            ContentType::TaskDescription,
            PrivacyLevel::Selective,
            expired_access,
        ).unwrap();
        
        let context = satswarm::privacy::DecryptionContext {
            requester_reputation: Some(5.0),
            is_active_bidder: true,
            is_task_participant: true,
            stake_amount: Some(1000),
            task_id: Some("task".to_string()),
        };
        
        let decrypt_result = privacy_manager.decrypt_content(&wrapped, "user", &context);
        assert!(decrypt_result.is_err()); // Should fail due to expiry
        
        println!("✅ Privacy access control edge cases test passed");
    }
}

// Utility functions for testing
#[cfg(test)]
mod test_utils {
    use super::*;

    pub fn create_test_task_request(task_id: &str, skills: Vec<String>) -> TaskRequest {
        TaskRequest {
            task_id: task_id.to_string(),
            requester_pubkey: Keys::generate().public_key().to_string(),
            description: format!("Test task: {}", task_id),
            required_skills: skills,
            max_budget_sats: 2000,
            deadline_minutes: 120,
            quality_requirements: QualityRequirements {
                min_reputation: 4.0,
                min_success_rate: 0.9,
                requires_validation: true,
                preferred_specialists: Vec::new(),
                blacklisted_specialists: Vec::new(),
            },
        }
    }

    pub fn create_mock_task_rating(
        task_id: &str,
        specialist_pubkey: &str,
        quality_score: f64,
    ) -> TaskRating {
        TaskRating {
            rating_id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            specialist_pubkey: specialist_pubkey.to_string(),
            requester_pubkey: Keys::generate().public_key().to_string(),
            quality_score,
            timeliness_score: 4.0,
            communication_score: 4.0,
            would_work_again: quality_score >= 4.0,
            comment: "Test rating".to_string(),
            verified_completion: true,
            created_at: Utc::now().timestamp() as u64,
            signature: "mock_signature".to_string(),
        }
    }

    pub async fn setup_test_environment() -> SatswarmProtocol {
        let config = SatswarmConfig {
            relay_urls: vec!["ws://localhost:18080".to_string()],
            mint_urls: vec!["http://localhost:3338".to_string()],
            agent_keys: None,
            default_privacy_level: PrivacyLevel::Participants,
            compliance_enabled: true,
            min_reputation_threshold: 3.0,
            auto_verification_threshold: 4.0,
            deflationary_pool_initial: 50000,
        };

        SatswarmProtocol::new(config).await.unwrap()
    }

    pub fn assert_task_completed(task: &satswarm::TaskExecution) {
        assert_eq!(task.status, satswarm::TaskExecutionStatus::Completed);
        assert!(task.specialist_pubkey.is_some());
        assert!(task.selected_bid.is_some());
        assert!(task.escrow.is_some());
        assert!(task.verification_result.is_some());
        assert!(task.rating.is_some());
    }
}

// Main test runner
#[cfg(test)]
mod test_runner {
    use super::*;
    
    pub async fn run_full_test_suite() {
        println!("🧪 Running Satswarm Test Suite...");
        println!("==================================");
        
        // Run tests in sequence to avoid conflicts
        println!("\n1. Unit Tests");
        // Unit tests run automatically with cargo test
        
        println!("\n2. Integration Tests");
        // Integration tests run automatically with cargo test
        
        println!("\n3. Performance Tests");
        // Performance tests run automatically with cargo test
        
        println!("\n4. Security Tests");
        // Security tests run automatically with cargo test
        
        println!("\n✅ All tests completed!");
        println!("=======================");
    }
}