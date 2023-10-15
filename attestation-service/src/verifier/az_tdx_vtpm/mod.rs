// Copyright (c) Microsoft Corporation.
//
// SPDX-License-Identifier: Apache-2.0
//

use super::tdx::claims::generate_parsed_claim;
use super::tdx::quote::{ecdsa_quote_verification, parse_tdx_quote, Quote as TdxQuote};
use super::{Attestation, TeeEvidenceParsedClaim, Verifier};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use az_tdx_vtpm::hcl::HclReport;
use az_tdx_vtpm::verify::Verify;
use az_tdx_vtpm::vtpm::Quote as TpmQuote;
use openssl::pkey::PKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};

#[derive(Serialize, Deserialize)]
struct Evidence {
    tpm_quote: TpmQuote,
    hcl_report: Vec<u8>,
    tdx_quote: Vec<u8>,
}

#[derive(Default)]
pub struct AzTdxVtpm;

// The verifier performs the following verification steps:
// 1. TDX Quote is genuine
// 2. TPM Quote has been signed by AK included in the HCL variable data
// 3. Attestation nonce matches TPM Quote nonce
// 4. TDX Quote report data matches hashed HCL variable data

#[async_trait]
impl Verifier for AzTdxVtpm {
    async fn evaluate(
        &self,
        nonce: String,
        attestation: &Attestation,
    ) -> Result<TeeEvidenceParsedClaim> {
        let evidence = serde_json::from_str::<Evidence>(&attestation.tee_evidence)?;
        let hashed_quote = nonced_pub_key_hash(attestation, &nonce);
        ecdsa_quote_verification(&evidence.tdx_quote).await?;
        let hcl_report = HclReport::new(evidence.hcl_report)?;
        verify_tpm_quote(&evidence.tpm_quote, &hcl_report, &hashed_quote)?;
        let tdx_quote = parse_tdx_quote(&evidence.tdx_quote)?;
        verify_report_data(&hcl_report, &tdx_quote)?;
        let claim = generate_parsed_claim(tdx_quote, None)?;
        Ok(claim)
    }
}

fn verify_report_data(hcl_report: &HclReport, tdx_quote: &TdxQuote) -> Result<()> {
    let var_data_hash = hcl_report.var_data_sha256();
    if var_data_hash != tdx_quote.report_body.report_data[..32] {
        return Err(anyhow!("TDX Quote report data mismatch"));
    }
    debug!("Report data verification completed successfully.");
    Ok(())
}

fn verify_tpm_quote(quote: &TpmQuote, hcl_report: &HclReport, hashed_nonce: &[u8]) -> Result<()> {
    let ak_pub = hcl_report.ak_pub()?;
    let pem = ak_pub.key.to_pem();
    let pub_key = PKey::public_key_from_pem(pem.as_bytes())?;
    quote.verify(&pub_key, hashed_nonce)?;
    debug!("TPM Quote verification completed successfully.");
    Ok(())
}

fn nonced_pub_key_hash(attestation: &Attestation, nonce: &str) -> Vec<u8> {
    let mut hasher = Sha384::new();
    hasher.update(nonce);
    hasher.update(&attestation.tee_pubkey.k_mod);
    hasher.update(&attestation.tee_pubkey.k_exp);
    hasher.finalize().to_vec()
}
