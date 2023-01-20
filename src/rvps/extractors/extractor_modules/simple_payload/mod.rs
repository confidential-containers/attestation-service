// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! Simple Payload is the format of reference values published by CoCo Community.

use std::collections::HashMap;

use anyhow::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rvps::{reference_value::primitive_date_time_from_str, ReferenceValue};

use super::Extractor;

#[derive(Serialize, Deserialize)]
pub struct Provenance {
    #[serde(flatten)]
    inner: HashMap<String, String>,
    #[serde(deserialize_with = "primitive_date_time_from_str")]
    expired: DateTime<Utc>,
}

#[derive(Default)]
pub struct SimplePayloadExtractor;

impl Extractor for SimplePayloadExtractor {
    fn verify_and_extract(&self, provenance: &str) -> Result<Vec<ReferenceValue>> {
        let provenance: Provenance = serde_json::from_str(provenance)?;
        let mut res = Vec::new();
        for (k, v) in &provenance.inner {
            let rv = ReferenceValue::new()?
                .set_version("1.0.0")
                .set_name(k)
                .set_expired(provenance.expired)
                .add_reference_value(v.clone());
            res.push(rv);
        }
        Ok(res)
    }
}
