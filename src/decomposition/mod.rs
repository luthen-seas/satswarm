// src/decomposition/mod.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    pub estimated_sats: u64,
    pub required_skills: Vec<String>,
    pub dependencies: Vec<String>,
    pub status: TaskStatus,
    pub subtasks: Vec<TaskNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Root,
    DataPreparation,
    Computation,
    Analysis,
    Visualization,
    Validation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Broadcast,
    Bidding,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTree {
    pub root: TaskNode,
    pub total_estimated_cost: u64,
    pub complexity_score: f64,
    pub estimated_duration_minutes: u32,
}

pub struct TaskDecomposer {
    domain_rules: HashMap<String, DecompositionRule>,
}

#[derive(Debug, Clone)]
struct DecompositionRule {
    keywords: Vec<String>,
    subtask_templates: Vec<SubTaskTemplate>,
    base_cost_sats: u64,
}

#[derive(Debug, Clone)]
struct SubTaskTemplate {
    description: String,
    task_type: TaskType,
    skills: Vec<String>,
    cost_multiplier: f64,
}

impl TaskDecomposer {
    pub fn new() -> Self {
        let mut domain_rules = HashMap::new();
        
        // Protein Analysis Domain
        domain_rules.insert("protein".to_string(), DecompositionRule {
            keywords: vec!["protein".to_string(), "structure".to_string(), "fold".to_string()],
            subtask_templates: vec![
                SubTaskTemplate {
                    description: "Protein sequence validation and preprocessing".to_string(),
                    task_type: TaskType::DataPreparation,
                    skills: vec!["bioinformatics".to_string(), "sequence_analysis".to_string()],
                    cost_multiplier: 0.2,
                },
                SubTaskTemplate {
                    description: "3D structure prediction and folding simulation".to_string(),
                    task_type: TaskType::Computation,
                    skills: vec!["molecular_dynamics".to_string(), "alphafold".to_string()],
                    cost_multiplier: 0.6,
                },
                SubTaskTemplate {
                    description: "Structure analysis and binding site identification".to_string(),
                    task_type: TaskType::Analysis,
                    skills: vec!["structural_biology".to_string(), "binding_analysis".to_string()],
                    cost_multiplier: 0.15,
                },
                SubTaskTemplate {
                    description: "Results visualization and report generation".to_string(),
                    task_type: TaskType::Visualization,
                    skills: vec!["pymol".to_string(), "scientific_visualization".to_string()],
                    cost_multiplier: 0.05,
                },
            ],
            base_cost_sats: 1000,
        });

        // Financial Analysis Domain
        domain_rules.insert("financial".to_string(), DecompositionRule {
            keywords: vec!["market".to_string(), "trading".to_string(), "portfolio".to_string(), "yield".to_string()],
            subtask_templates: vec![
                SubTaskTemplate {
                    description: "Market data collection and cleaning".to_string(),
                    task_type: TaskType::DataPreparation,
                    skills: vec!["data_engineering".to_string(), "market_data".to_string()],
                    cost_multiplier: 0.3,
                },
                SubTaskTemplate {
                    description: "Quantitative analysis and modeling".to_string(),
                    task_type: TaskType::Computation,
                    skills: vec!["quantitative_analysis".to_string(), "risk_modeling".to_string()],
                    cost_multiplier: 0.5,
                },
                SubTaskTemplate {
                    description: "Performance metrics and risk assessment".to_string(),
                    task_type: TaskType::Analysis,
                    skills: vec!["portfolio_analysis".to_string(), "risk_management".to_string()],
                    cost_multiplier: 0.2,
                },
            ],
            base_cost_sats: 800,
        });

        // Machine Learning Domain
        domain_rules.insert("ml".to_string(), DecompositionRule {
            keywords: vec!["machine learning".to_string(), "model".to_string(), "training".to_string(), "predict".to_string()],
            subtask_templates: vec![
                SubTaskTemplate {
                    description: "Dataset preparation and feature engineering".to_string(),
                    task_type: TaskType::DataPreparation,
                    skills: vec!["data_science".to_string(), "feature_engineering".to_string()],
                    cost_multiplier: 0.25,
                },
                SubTaskTemplate {
                    description: "Model training and hyperparameter optimization".to_string(),
                    task_type: TaskType::Computation,
                    skills: vec!["machine_learning".to_string(), "gpu_computing".to_string()],
                    cost_multiplier: 0.55,
                },
                SubTaskTemplate {
                    description: "Model evaluation and validation".to_string(),
                    task_type: TaskType::Validation,
                    skills: vec!["model_validation".to_string(), "statistical_analysis".to_string()],
                    cost_multiplier: 0.2,
                },
            ],
            base_cost_sats: 1200,
        });

        Self { domain_rules }
    }

    pub fn decompose(&self, user_request: &str) -> Result<TaskTree, String> {
        let request_lower = user_request.to_lowercase();
        
        // Determine domain based on keywords
        let domain = self.detect_domain(&request_lower)?;
        let rule = self.domain_rules.get(&domain)
            .ok_or_else(|| format!("No decomposition rule found for domain: {}", domain))?;

        // Create root task
        let root_id = Uuid::new_v4().to_string();
        let mut subtasks = Vec::new();
        let mut total_cost = 0u64;

        // Generate subtasks based on templates
        for template in &rule.subtask_templates {
            let subtask_cost = (rule.base_cost_sats as f64 * template.cost_multiplier) as u64;
            total_cost += subtask_cost;

            subtasks.push(TaskNode {
                id: Uuid::new_v4().to_string(),
                description: template.description.clone(),
                task_type: template.task_type.clone(),
                estimated_sats: subtask_cost,
                required_skills: template.skills.clone(),
                dependencies: Vec::new(),
                status: TaskStatus::Pending,
                subtasks: Vec::new(),
            });
        }

        // Set up dependencies (each task depends on the previous one)
        for i in 1..subtasks.len() {
            subtasks[i].dependencies.push(subtasks[i-1].id.clone());
        }

        let root = TaskNode {
            id: root_id,
            description: user_request.to_string(),
            task_type: TaskType::Root,
            estimated_sats: total_cost,
            required_skills: Vec::new(),
            dependencies: Vec::new(),
            status: TaskStatus::Pending,
            subtasks,
        };

        // Calculate complexity score based on number of subtasks and skills
        let complexity_score = self.calculate_complexity(&root);
        let estimated_duration = self.estimate_duration(&root);

        Ok(TaskTree {
            root,
            total_estimated_cost: total_cost,
            complexity_score,
            estimated_duration_minutes: estimated_duration,
        })
    }

    fn detect_domain(&self, request: &str) -> Result<String, String> {
        let mut best_match = None;
        let mut best_score = 0;

        for (domain, rule) in &self.domain_rules {
            let score = rule.keywords.iter()
                .filter(|keyword| request.contains(keyword))
                .count();
            
            if score > best_score {
                best_score = score;
                best_match = Some(domain.clone());
            }
        }

        best_match.ok_or_else(|| {
            "Could not determine domain. Supported domains: protein, financial, ml".to_string()
        })
    }

    fn calculate_complexity(&self, root: &TaskNode) -> f64 {
        let subtask_count = root.subtasks.len() as f64;
        let unique_skills: std::collections::HashSet<_> = root.subtasks.iter()
            .flat_map(|task| &task.required_skills)
            .collect();
        let skill_diversity = unique_skills.len() as f64;
        
        // Complexity score: weighted average of subtask count and skill diversity
        (subtask_count * 0.6 + skill_diversity * 0.4) / 10.0
    }

    fn estimate_duration(&self, root: &TaskNode) -> u32 {
        // Simple estimation: 30 minutes base + 15 minutes per subtask
        30 + (root.subtasks.len() as u32 * 15)
    }
}

// NOSTR integration
use nostr::{Event, EventBuilder, Keys, Kind, Tag};

impl TaskTree {
    pub fn to_nostr_event(&self, keys: &Keys) -> Result<Event, Box<dyn std::error::Error>> {
        let content = serde_json::to_string(self)?;
        let tags = vec![
            Tag::generic(nostr::TagKind::Custom("task_type".into()), vec!["decomposition"]),
            Tag::generic(nostr::TagKind::Custom("estimated_cost".into()), vec![&self.total_estimated_cost.to_string()]),
            Tag::generic(nostr::TagKind::Custom("complexity".into()), vec![&format!("{:.2}", self.complexity_score)]),
        ];

        let event = EventBuilder::new(Kind::Custom(30001), &content, tags)
            .sign_with_keys(keys)?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protein_decomposition() {
        let decomposer = TaskDecomposer::new();
        let result = decomposer.decompose("Analyze the structure of protein BRCA1").unwrap();
        
        assert_eq!(result.root.subtasks.len(), 4);
        assert_eq!(result.root.task_type, TaskType::Root);
        assert!(result.total_estimated_cost > 0);
        
        // Check that first subtask is data preparation
        assert_eq!(result.root.subtasks[0].task_type, TaskType::DataPreparation);
    }

    #[test]
    fn test_ml_decomposition() {
        let decomposer = TaskDecomposer::new();
        let result = decomposer.decompose("Train a machine learning model to predict stock prices").unwrap();
        
        assert_eq!(result.root.subtasks.len(), 3);
        assert!(result.total_estimated_cost >= 1200);
    }

    #[test]
    fn test_unknown_domain() {
        let decomposer = TaskDecomposer::new();
        let result = decomposer.decompose("Cook a delicious pasta");
        
        assert!(result.is_err());
    }
}