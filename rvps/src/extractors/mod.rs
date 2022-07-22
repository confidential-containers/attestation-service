// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! Extractors for RVPS.

pub mod extractor_modules;

use anyhow::*;

use std::collections::HashMap;

use crate::{reference_value::ReferenceValue, Message};

use self::extractor_modules::{ExtractorInstance, ExtractorModuleList};

/// `Extractors` provides different kinds of `Extractor`s due to
/// different provenance types, e.g. in-toto, etc.
/// Each `Extractor` will process the input provenance, verify the
/// authenticity of the provenance, and then extract the formatted
/// reference value (degest, s.t. hash value and name of the artifact)
/// from the provenance. If the verification fails, no reference value
/// will be extracted.

/// `ExtractorsAPI` defines the interfaces of Extractors.
pub trait ExtractorsAPI {
    /// Process the message, e.g. verifying
    /// and extracting the provenance inside the message due to
    /// type also inside the same message. If verification
    /// succeeds, return the generated ReferenceValue.
    fn process(&mut self, message: Message) -> Result<ReferenceValue>;
}

/// The struct `Extractors` is responsible for implementing
/// trait `Extractors`.
/// `extractors_module_list` is a map that maps provenance type
/// (in String) to its Extractor's instancializer.
/// `extractors_instance_map` is another map that maps provenance type
/// to the instancialized Extractor. The two map implement a
/// "instantialization-on-demand" mechanism.
pub struct Extractors {
    extractors_module_list: ExtractorModuleList,
    extractors_instance_map: HashMap<String, ExtractorInstance>,
}

impl Default for Extractors {
    fn default() -> Self {
        Self::new()
    }
}

impl Extractors {
    pub fn new() -> Self {
        let extractors_module_list = ExtractorModuleList::new();
        let extractors_instance_map = HashMap::new();
        Extractors {
            extractors_module_list,
            extractors_instance_map,
        }
    }

    /// Register an `Extractor` instance to `Extractors`. The `Extractor` is responsible for
    /// handling specific kind of provenance (as `extractor_name` indicates).
    fn register_instance(&mut self, extractor_name: String, extractor_instance: ExtractorInstance) {
        self.extractors_instance_map
            .insert(extractor_name, extractor_instance);
    }

    /// Instantiate an `Extractor` of given type `extractor_name`. This method will
    /// instantiate an `Extractor` instance and then register it.
    fn instantiate_extractor(&mut self, extractor_name: String) -> Result<()> {
        let instantiate_func = self.extractors_module_list.get_func(&extractor_name)?;
        let extractor_instance = (instantiate_func)();
        self.register_instance(extractor_name, extractor_instance);
        Ok(())
    }
}

impl ExtractorsAPI for Extractors {
    fn process(&mut self, message: Message) -> Result<ReferenceValue> {
        let typ = message.typ;

        if self.extractors_instance_map.get_mut(&typ).is_none() {
            self.instantiate_extractor(typ.clone())?;
        }
        let extractor_instance = self
            .extractors_instance_map
            .get_mut(&typ)
            .ok_or_else(|| anyhow!("The Extractor instance does not existing!"))?;

        extractor_instance.verify_and_extract(&message.payload)
    }
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};
    use serial_test::serial;

    use crate::{
        extractors::extractor_modules::in_toto::test::sha256_for_in_toto_test_artifact, Message,
        ReferenceValue, MESSAGE_VERSION,
    };

    use super::{
        extractor_modules::in_toto::test::generate_in_toto_provenance, Extractors, ExtractorsAPI,
    };

    #[test]
    #[serial]
    fn extractors_using_in_toto() {
        let mut e = Extractors::new();
        let in_toto_provenance = generate_in_toto_provenance();
        let message = Message {
            version: MESSAGE_VERSION.to_string(),
            payload: in_toto_provenance,
            typ: "in-toto".into(),
        };
        let res = e.process(message).unwrap();
        let rv = ReferenceValue::new()
            .set_name("foo.tar.gz")
            .set_expired(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0))
            .set_version("0.1")
            .add_hash_value("sha256".into(), sha256_for_in_toto_test_artifact());

        assert_eq!(res, rv);
    }
}
