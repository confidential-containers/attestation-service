// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

//! # in-toto Extractor
//!
//! This Extractor helps to verify in-toto metadata and extract
//! related reference value from link file.

use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{create_dir_all, File},
    io::Write,
};

use anyhow::{anyhow, bail, Result};
use in_toto::models::{LayoutMetadata, TargetDescription, VirtualTargetPath};
use in_totolib_rs::intoto::verify;
use serde::{Deserialize, Serialize};

use crate::reference_value::{ReferenceValue, REFERENCE_VALUE_VERSION};

use super::Extractor;

static INTOTO_VERSION: &str = "0.9";

#[derive(Serialize, Deserialize)]
pub struct Provenance {
    #[serde(default = "default_version")]
    version: String,
    line_normalization: bool,
    files: HashMap<String, String>,
}

/// Use to set default version of Provenance
fn default_version() -> String {
    INTOTO_VERSION.into()
}

/// payload in Reference Value
#[derive(Serialize, Deserialize)]
pub struct Payload {
    artifacts: BTreeMap<VirtualTargetPath, TargetDescription>,
}

pub struct InTotoExtractor;

impl InTotoExtractor {
    pub fn new() -> Self {
        InTotoExtractor
    }
}

impl Extractor for InTotoExtractor {
    /// In-toto's Extractor.
    /// It needs the following parameters in the HashMap:
    /// * `layout_path`: path to the layout file.
    /// * `pub_key_paths`: serialized json string of a Vec, including
    /// paths of public keys.
    /// * `intermediate_paths`: serialized json string of a Vec, including
    /// paths of intermediate.
    /// * `link_dir`: path to the directory of link files.
    /// * `line_normalization`: whether line normalization is enabled (true
    /// or false), which means whether Windows-style line separators
    /// (CRLF) are normalized to Unix-style line separators (LF) for
    /// cross-platform consistency.
    fn verify_and_extract(&self, provenance: &str) -> Result<Vec<ReferenceValue>> {
        // Deserialize Provenance
        let payload: Provenance = serde_json::from_str(provenance)?;

        // Judge version
        if payload.version != INTOTO_VERSION {
            return Err(anyhow!(
                "Version unmatched! Need {}, given {}.",
                INTOTO_VERSION,
                payload.version
            ));
        }

        // Create tempdir and put the files
        let tempdir = tempfile::tempdir()?;
        let tempdir_path = tempdir.into_path();

        (&payload.files)
            .iter()
            .try_for_each(|(relative_path, content_base64)| -> Result<()> {
                let mut file_path = tempdir_path.clone();
                file_path.push(relative_path);
                let dir_path = file_path
                    .parent()
                    .ok_or_else(|| anyhow!("In-toto get file parent path failed."))?;

                create_dir_all(dir_path)?;
                let mut file = File::create(file_path)?;
                let bytes = base64::decode(content_base64)?;
                file.write_all(&bytes)?;
                Ok(())
            })?;

        // get link dir (temp dir)
        let link_dir = tempdir_path.to_string_lossy().to_string();

        // get layout file
        let layout_name = payload
            .files
            .keys()
            .find(|&k| k.ends_with(".layout"))
            .ok_or_else(|| anyhow!("Layout file not found."))?
            .to_owned();

        let mut layout_path_buf = tempdir_path.clone();
        layout_path_buf.push(layout_name);
        let layout_path = layout_path_buf.to_string_lossy().to_string();

        // get pub keys
        let pub_key_paths: Vec<String> = {
            let file_names: Vec<String> = payload
                .files
                .keys()
                .filter_map(|k| match k.ends_with(".pub") {
                    true => Some(k.to_string()),
                    false => None,
                })
                .collect();
            let mut file_paths = Vec::new();
            for file_name in file_names {
                let mut key_path_buf = tempdir_path.clone();
                key_path_buf.push(file_name);
                let key_path = key_path_buf.to_string_lossy().to_string();
                file_paths.push(key_path);
            }
            file_paths
        };

        let intermediate_paths = Vec::new();

        let line_normalization = payload.line_normalization;

        // Store and change current dir to the tmp dir
        let cwd = env::current_dir()?;

        // Read layout for the expired time
        let layout_file = std::fs::File::open(&layout_path)?;
        let layout: LayoutMetadata = serde_json::from_reader(layout_file)?;

        env::set_current_dir(tempdir_path)?;

        // Here the returned value is "" when verification successeds
        let summary_link = match verify(
            layout_path,
            pub_key_paths,
            intermediate_paths,
            link_dir,
            line_normalization,
        ) {
            Ok(summary_link) => summary_link,
            Err(e) => {
                env::set_current_dir(cwd)?;
                bail!(e);
            }
        };

        // Change back working dir
        env::set_current_dir(cwd)?;

        let mut res = Vec::new();

        for (filepath, digests) in summary_link.products() {
            let filename = filepath
                .value()
                .split('/')
                .last()
                .ok_or_else(|| anyhow!("Unexpected empty product name"))?;
            let mut rv = ReferenceValue::new()?
                .set_name(filename)
                .set_version(REFERENCE_VALUE_VERSION)
                .set_expired(*layout.expires());

            for (alg, value) in digests {
                let alg = serde_json::to_string(alg)?
                    .trim_end_matches('\"')
                    .trim_start_matches('\"')
                    .to_string();

                let value = serde_json::to_string(value)?
                    .trim_end_matches('\"')
                    .trim_start_matches('\"')
                    .to_string();

                rv = rv.add_hash_value(alg.to_string(), value.to_string());
            }

            res.push(rv);
        }
        Ok(res)
    }
}

#[cfg(test)]
pub mod test {
    use std::{collections::HashMap, fs};

    use chrono::{TimeZone, Utc};
    use serial_test::serial;
    use sha2::{Digest, Sha256};
    use walkdir::WalkDir;

    use crate::{extractors::extractor_modules::Extractor, ReferenceValue};

    use super::{InTotoExtractor, Provenance, INTOTO_VERSION};

    /// Helps to get sha256 digest of the artifact
    pub fn sha256_for_in_toto_test_artifact() -> String {
        println!("{:?}", std::env::current_dir());
        let content = fs::read("tests/in-toto/foo.tar.gz").unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let result = format!("{:x}", result);
        result
    }

    /// Helps to generate a in-toto provenance encoded
    /// in Base64. All related files are in `<git-repo>/tests/in-toto`
    pub fn generate_in_toto_provenance() -> String {
        let mut files = HashMap::new();

        for path in WalkDir::new("tests/in-toto") {
            let path = path.unwrap();
            if path.file_type().is_dir() {
                continue;
            }

            let ent = path.path();
            let content = fs::read(&ent).unwrap();
            let file_name = ent
                .to_string_lossy()
                .to_string()
                .strip_prefix("tests/in-toto")
                .expect("failed to strip prefix")
                .into();
            let content_base64 = base64::encode(content);

            // split_off will delete the prefix "../tests/in-toto" for every
            // file
            files.insert(file_name, content_base64);
        }

        let p = Provenance {
            version: INTOTO_VERSION.into(),
            line_normalization: true,
            files,
        };

        let provenance = serde_json::to_string(&p).unwrap();
        provenance
    }

    #[test]
    #[serial]
    fn in_toto_extractor() {
        let e = InTotoExtractor::new();
        let rv = ReferenceValue::new()
            .expect("create ReferenceValue failed")
            .set_name("foo.tar.gz")
            .set_expired(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap())
            .set_version("0.1")
            .add_hash_value("sha256".into(), sha256_for_in_toto_test_artifact());
        let rv = vec![rv];
        let provenance = generate_in_toto_provenance();
        let res = e.verify_and_extract(&provenance).unwrap();

        assert_eq!(res, rv);
    }
}
