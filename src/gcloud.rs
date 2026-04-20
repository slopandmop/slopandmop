// Typed wrapper over `gcloud compute …` shelling out. We deliberately do NOT
// pull in google-cloud-compute for MVP; the shell pattern is what dd already
// uses and it keeps the dep graph small. See:
//   /home/tdx2/src/dd/.github/workflows/deploy-cp.yml:146-157  (create)
//   /home/tdx2/src/dd/.github/workflows/deploy-cp.yml:177-207  (health poll)

use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

pub struct CreateArgs<'a> {
    pub project: &'a str,
    pub zone: &'a str,
    pub vm_name: &'a str,
    pub image_family: &'a str,
    pub machine_type: &'a str,
    pub ee_config_path: &'a Path,
}

pub async fn create(args: CreateArgs<'_>) -> Result<()> {
    let status = Command::new("gcloud")
        .args([
            "compute",
            "instances",
            "create",
            args.vm_name,
            "--project",
            args.project,
            "--zone",
            args.zone,
            "--machine-type",
            args.machine_type,
            "--confidential-compute-type=TDX",
            "--maintenance-policy=TERMINATE",
            "--image-family",
            args.image_family,
            "--image-project",
            args.project,
            "--metadata-from-file",
        ])
        .arg(format!("ee-config={}", args.ee_config_path.display()))
        .status()
        .await
        .context("failed to spawn gcloud")?;
    if !status.success() {
        anyhow::bail!("gcloud compute instances create exited {status}");
    }
    Ok(())
}

pub async fn delete(project: &str, zone: &str, vm_name: &str) -> Result<()> {
    let status = Command::new("gcloud")
        .args([
            "compute",
            "instances",
            "delete",
            vm_name,
            "--project",
            project,
            "--zone",
            zone,
            "--quiet",
        ])
        .status()
        .await
        .context("failed to spawn gcloud")?;
    if !status.success() {
        anyhow::bail!("gcloud compute instances delete exited {status}");
    }
    Ok(())
}
