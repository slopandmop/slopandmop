// sm-register — one-shot. Runs before the other sm boot workloads.
//
// 1. Fetch ITA token from configfs-tsm (see easyenclave/src/attestation/tsm.rs:54-81).
// 2. POST {sm_id, ita_token, hostname} to $SM_FACTORY_URL/sm/register.
// 3. Write {tunnel_token, hostnames} to /var/lib/easyenclave/run/registered.
// 4. Exit 0. Other workloads `until`-loop on the registered file.

use anyhow::Result;

pub async fn run() -> Result<()> {
    let cfg = crate::config::SmEnv::from_env()?;
    tracing::info!(sm_id = %cfg.sm_id, factory = %cfg.factory_url, "sm-register starting");

    // TODO:
    // let token = read_ita_token().await?;
    // let resp = reqwest::Client::new()
    //     .post(format!("{}/sm/register", cfg.factory_url))
    //     .json(&json!({
    //         "sm_id": cfg.sm_id,
    //         "ita_token": token,
    //         "hostname": cfg.hostnames.sm,
    //     }))
    //     .send().await?.error_for_status()?.json::<Value>().await?;
    // std::fs::create_dir_all("/var/lib/easyenclave/run")?;
    // std::fs::write("/var/lib/easyenclave/run/registered", serde_json::to_vec(&resp)?)?;

    anyhow::bail!("register::run not yet implemented")
}
