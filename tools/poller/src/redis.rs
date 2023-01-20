// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use anyhow::*;
use redis::{Client, Commands};
use serde::Serialize;

/// Redis channel to publish
const CHANNEL: &str = "provenance";

/// Message is an overall packet that Reference Value Provider Service
/// receives. It will contain payload (content of different provenance,
/// JSON format), provenance type (indicates the type of the payload)
/// and a version number (use to distinguish different version of
/// message, for extendability).
/// * `version`: version of this message.
/// * `payload`: content of the provenance, JSON encoded.
/// * `type`: provenance type of the payload.
#[derive(Serialize, Debug)]
pub struct Message {
    version: String,
    payload: String,
    r#type: String,
}

/// Publish the provenance to subscriber via redis
pub async fn publish(provenance: &str, url: &str) -> Result<()> {
    let message = Message {
        version: "0.1.0".into(),
        payload: provenance.into(),
        r#type: "simple-payload".to_string(),
    };

    publish_message(message, url).await
}

async fn publish_message(message: Message, url: &str) -> Result<()> {
    let client: Client = redis::Client::open(url)?;
    let mut conn = client.get_connection()?;

    let json = serde_json::to_string(&message)?;
    conn.publish(CHANNEL, json)?;
    Ok(())
}
