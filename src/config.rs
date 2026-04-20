// Environment-variable config read from ee-config (GCE metadata overlay).
//
// Factory and sm instances see different shapes. Each mode picks the subset
// it needs. Missing fields produce a clear error rather than silent defaults.

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct FactoryEnv {
    pub factory_url: String,
    pub gcp_project: String,
    pub gcp_zone: String,
    pub sm_image_family: String,
    pub child_image_family: String,
    pub anthropic_api_key: String,
    pub cloudflare_token: String,
    pub cloudflare_zone_id: String,
    pub registry_path: std::path::PathBuf,
}

impl FactoryEnv {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            factory_url: req("FACTORY_URL")?,
            gcp_project: req("GCP_PROJECT")?,
            gcp_zone: req("GCP_ZONE")?,
            sm_image_family: opt("SM_IMAGE_FAMILY", "slopandmop-stable"),
            child_image_family: opt("CHILD_IMAGE_FAMILY", "easyenclave-stable"),
            anthropic_api_key: req("ANTHROPIC_API_KEY")?,
            cloudflare_token: req("CF_API_TOKEN")?,
            cloudflare_zone_id: req("CF_ZONE_ID")?,
            registry_path: opt("REGISTRY_PATH", "/var/lib/easyenclave/factory/registry.json").into(),
        })
    }
}

#[derive(Debug)]
pub struct SmEnv {
    pub sm_id: String,
    pub sm_cookie_key: String,
    pub factory_url: String,
    pub hostnames: SmHostnames,
}

#[derive(Debug)]
pub struct SmHostnames {
    pub sm: String,
    pub chat: String,
}

impl SmEnv {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            sm_id: req("SM_ID")?,
            sm_cookie_key: req("SM_COOKIE_KEY")?,
            factory_url: req("SM_FACTORY_URL")?,
            hostnames: SmHostnames {
                sm: req("SM_HOSTNAME_SM")?,
                chat: req("SM_HOSTNAME_CHAT")?,
            },
        })
    }
}

fn req(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("env {key} is required"))
}

fn opt(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
