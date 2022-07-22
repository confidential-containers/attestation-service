// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! A simple cache. It only stores rv in the memory. When
//! the core object is destroyed, all the stored rv will be
//! lost.

use std::collections::HashMap;

use anyhow::Result;

use crate::reference_value::ReferenceValue;

use super::Cache;

pub struct SimpleCache {
    inner: HashMap<String, ReferenceValue>,
}

impl Cache for SimpleCache {
    fn set(&mut self, name: String, rv: ReferenceValue) -> Result<()> {
        self.inner.insert(name, rv);
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Option<ReferenceValue>> {
        Ok(self.inner.get(name).cloned())
    }
}

impl SimpleCache {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}
