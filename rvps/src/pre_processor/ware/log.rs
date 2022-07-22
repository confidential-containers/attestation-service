// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! A simple log ware for pre-processor

use std::collections::HashMap;

use anyhow::*;
use log::info;

use crate::{
    pre_processor::{Next, Ware},
    Message,
};

/// A LogWare can log the received provenance to terminal
pub struct LogWare {}

impl LogWare {
    pub fn new() -> Self {
        Self {}
    }
}

impl Ware for LogWare {
    fn handle(
        &self,
        message: &mut Message,
        context: &mut HashMap<String, String>,
        next: Next<'_>,
    ) -> Result<()> {
        info!("Get a new provenance of type: {}", message.typ);
        next.run(message, context)
    }
}
