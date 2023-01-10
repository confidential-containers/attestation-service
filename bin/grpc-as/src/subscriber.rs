use std::sync::Arc;

use anyhow::*;
use attestation_service::{rvps::Message, AttestationService};
use tokio::sync::RwLock;

const CHANNEL: &str = "provenance";

pub async fn subscribe(redis_url: String, service: Arc<RwLock<AttestationService>>) -> Result<()> {
    let client = redis::Client::open(redis_url).context("open redis client failed")?;
    let mut conn = client
        .get_connection()
        .context("connect redis to subscribe failed")?;

    let mut sub = conn.as_pubsub();
    sub.subscribe(CHANNEL).context("subscribe failed")?;
    loop {
        let msg = sub.get_message()?;
        let msg = msg.get_payload_bytes();
        let message: Message = serde_json::from_slice(msg)?;
        {
            let mut writer = service.write().await;
            writer
                .registry_reference_value(message)
                .await
                .context("register reference value failed: {e}")?;
        }
    }
}
