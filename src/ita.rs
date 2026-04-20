// Verify Intel Trust Authority (ITA) tokens presented by sm and child VMs.
//
// Lift the shape from /home/tdx2/src/dd/src/cp.rs:318-425 — JWT issued by
// the TDX runtime (see easyenclave/src/attestation/tsm.rs:54-81), claims
// include the MRTD measurement + a caller-controlled report_data_b64 we
// bind to sm_id. Factory checks:
//   1. JWT signature against Intel's JWKS.
//   2. `iss` matches the ITA issuer URL.
//   3. `exp` + freshness window (re-prove every 180 s).
//   4. report_data_b64 decodes to the sm_id it's being used for.
//   5. sm_id exists in factory's registry (rejects post-teardown calls).

use anyhow::{bail, Result};

/// Extracted ITA claims — the shape we care about in factory.
#[derive(Debug)]
pub struct Claims {
    pub sm_id: String,
    pub mrtd: String,
    pub exp: i64,
}

pub async fn verify(_token: &str) -> Result<Claims> {
    // TODO: port dd::cp ITA verification here. See dd/src/cp.rs:318-425.
    bail!("ita::verify not yet implemented")
}
