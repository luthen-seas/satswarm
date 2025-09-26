//! Simple Satswarm Agent Example

use satswarm::{SatswarmConfig, SatswarmProtocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Simple Satswarm Agent Example");

    let config = SatswarmConfig::default();
    let protocol = SatswarmProtocol::new(config).await?;

    println!("✅ Agent initialized successfully!");
    println!("🔗 Ready to process tasks...");

    let task_id = protocol
        .process_task_request("Example task: Analyze sample data", 1000, 60)
        .await?;

    println!("🎉 Task submitted with ID: {}", task_id);

    Ok(())
}
