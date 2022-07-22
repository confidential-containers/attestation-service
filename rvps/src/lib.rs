// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

#![allow(clippy::new_without_default)]

pub mod cache;
pub mod extractors;
pub mod pre_processor;
pub mod reference_value;

use anyhow::{anyhow, Result};
use extractors::{Extractors, ExtractorsAPI};
use pre_processor::{PreProcessor, PreProcessorAPI, Ware};
use serde::{Deserialize, Serialize};

pub use cache::Cache;
pub use pre_processor::ware;
pub use reference_value::ReferenceValue;

/// Default version of Message
static MESSAGE_VERSION: &str = "0.1";

/// Message is an overall packet that Reference Value Provider Service
/// receives. It will contain payload (content of different provenance,
/// JSON format), provenance type (indicates the type of the payload)
/// and a version number (use to distinguish different version of
/// message, for extendability).
/// * `version`: version of this message.
/// * `payload`: content of the provenance, JSON encoded.
/// * `typ`: provenance type of the payload.
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(default = "default_version")]
    version: String,
    payload: String,
    #[serde(rename = "type")]
    typ: String,
}

/// Set the default version for Message
fn default_version() -> String {
    MESSAGE_VERSION.into()
}

/// The interfaces of Reference Value Provider Service
/// * `verify_and_extract` is responsible for verify a message and
/// store reference values from it.
/// * `get_rv` gets rv by the artifact's name.
pub trait RVPSAPI {
    fn verify_and_extract(&mut self, message: Message) -> Result<()>;
    fn get_rv(&self, name: &str) -> Result<Option<ReferenceValue>>;
}

/// The core of the RVPS, s.t. componants except communication componants.
pub struct Core<T: Cache> {
    pre_processor: PreProcessor,
    extractors: Extractors,
    cache: T,
}

impl<T: Cache> Core<T> {
    /// Instantialize a new RVPS Core
    pub fn new(cache: T) -> Self {
        let pre_processor = PreProcessor::new();

        let extractors = Extractors::new();

        Core {
            pre_processor,
            extractors,
            cache,
        }
    }

    /// Add Ware to the Core's Pre-Processor
    pub fn with_ware(&mut self, ware: Box<dyn Ware>) -> &Self {
        self.pre_processor.add_ware(ware);
        self
    }
}

impl<T: Cache> RVPSAPI for Core<T> {
    fn verify_and_extract(&mut self, mut message: Message) -> Result<()> {
        // Judge the version field
        if message.version != MESSAGE_VERSION {
            return Err(anyhow!(
                "Version unmatched! Need {}, given {}.",
                MESSAGE_VERSION,
                message.version
            ));
        }

        self.pre_processor.process(&mut message)?;

        let rv = self.extractors.process(message)?;
        self.cache.set(rv.name().to_string(), rv)?;
        Ok(())
    }

    fn get_rv(&self, name: &str) -> Result<Option<ReferenceValue>> {
        self.cache.get(name)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use log::Level;
    use serial_test::serial;

    use crate::{
        cache::simple::SimpleCache,
        extractors::extractor_modules::in_toto::test::{
            generate_in_toto_provenance, sha256_for_in_toto_test_artifact,
        },
        pre_processor::ware::log::LogWare,
        Core, Message, ReferenceValue, MESSAGE_VERSION, RVPSAPI,
    };

    extern crate testing_logger;

    #[test]
    #[serial]
    fn test_core() {
        let mut core = Core::new(SimpleCache::new());
        let message = Message {
            version: MESSAGE_VERSION.into(),
            typ: "in-toto".into(),
            payload: generate_in_toto_provenance(),
        };
        core.verify_and_extract(message).unwrap();
        let rv = ReferenceValue::new()
            .set_name("foo.tar.gz")
            .set_expired(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0))
            .set_version("0.1")
            .add_hash_value("sha256".into(), sha256_for_in_toto_test_artifact());
        let res = core.get_rv("foo.tar.gz").unwrap();
        assert_eq!(res, Some(rv));
    }

    #[test]
    #[serial]
    fn test_core_with_ware() {
        testing_logger::setup();
        let mut core = Core::new(SimpleCache::new());
        core.with_ware(Box::new(LogWare::new()));
        let message = Message {
            version: MESSAGE_VERSION.into(),
            typ: "in-toto".into(),
            payload: generate_in_toto_provenance(),
        };
        core.verify_and_extract(message).unwrap();

        testing_logger::validate(|captured_logs| {
            assert_eq!(
                captured_logs[0].body,
                "Get a new provenance of type: in-toto"
            );
            assert_eq!(captured_logs[0].level, Level::Info);
        });
    }
}
