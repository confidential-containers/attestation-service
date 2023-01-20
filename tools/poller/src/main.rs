// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! Poller

use std::time::Duration;

use anyhow::*;
use clap::{arg, command};
use lazy_static::lazy_static;
use log::{debug, info};
use reqwest::Url;
use semver::Version;
use tokio::sync::Mutex;

use crate::redis::publish;

mod redis;

/// Reference Values repository set up by CoCo community
const REFERENCE_VALUE_REPO: &str =
    "https://github.com/confidential-containers/provenance/blob/main/";

/// The file under the repo to record the latest reference value version
const LATEST_REFERENCE_VERSION_FILE: &str = "latest";

/// Interval of seconds between two access
const DEFAULT_INTERVAL: &str = "120";

/// Default redis url
const DEFAULT_REDIS_URL: &str = "redis://localhost/";

lazy_static! {
    static ref LATEST_VERSION: Mutex<Version> =
        Mutex::new(Version::parse("0.0.0").expect("failed to parse initial version"));
}

/// Get the latest version of the reference values
/// from the given repo. If the retrieved reference value is the same
/// as the current one, it will wait by default [`INTERVAL`] seconds and retry.
async fn fetch_latest_version(repo: &str, interval: u64) -> Result<String> {
    let url = Url::parse(repo)?;
    url.join(LATEST_REFERENCE_VERSION_FILE)?;
    let duration = Duration::from_secs(interval);

    loop {
        let response = reqwest::get(url.clone()).await?;
        let version = response.text().await?;
        let new_ver = Version::parse(&version)?;
        let new_ver_str = new_ver.to_string();
        if new_ver <= *LATEST_VERSION.lock().await {
            debug!("same version of reference value found {new_ver_str}, wait for {interval} seconds to retry...");
            tokio::time::sleep(duration).await;
            continue;
        }

        // got new version

        {
            let mut lv = LATEST_VERSION.lock().await;
            *lv = new_ver;
        }

        info!("find new reference value version {new_ver_str}.");
        return Ok(new_ver_str);
    }
}

/// Fetch the reference value from the given repo of the specific version
async fn fetch_reference_value(repo: &str, version: &str) -> Result<String> {
    let url = Url::parse(repo)?;
    url.join(version)?;

    debug!("try to fetch reference value from {} ...", url.as_str());
    let response = reqwest::get(url).await?;
    let provenance = response.text().await?;

    debug!("succeed fetching reference value.");
    Ok(provenance)
}

async fn true_main() -> Result<()> {
    let cmd = command!("Poller")
        .arg(
            arg!(-r --"repo-url" "repository to store the reference data")
                .required(false)
                .default_value(REFERENCE_VALUE_REPO),
        )
        .arg(
            arg!(-i --interval "interval between two polls (seconds)")
                .required(false)
                .default_value(DEFAULT_INTERVAL),
        )
        .arg(
            arg!(-u --redis "url of redis")
                .required(false)
                .default_value(DEFAULT_REDIS_URL),
        )
        .get_matches();

    let repo = cmd.value_of("repo-url").expect("no repo is given");
    let interval = cmd
        .value_of("interval")
        .expect("no interval is given")
        .parse::<u64>()?;

    let redis_url = cmd.value_of("redis").expect("no redis-url is given");
    loop {
        let new_ver = fetch_latest_version(repo, interval).await?;
        let reference_value = fetch_reference_value(repo, &new_ver).await?;

        publish(&reference_value, redis_url).await?;
        info!("provenance published: {reference_value}");
    }
}

#[tokio::main]
async fn main() {
    match true_main().await {
        Err(e) => eprintln!("[Error]: {e}"),
        _ => unreachable!(),
    }
}
