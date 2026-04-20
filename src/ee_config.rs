// Bake the `ee-config` blob we pass to `gcloud instances create
// --metadata-from-file=ee-config=...`.
//
// ee reads this at boot (see easyenclave/src/init.rs `fetch_gce_metadata_config`)
// and applies each key as an env var. Key env vars we set:
//
//   EE_BOOT_WORKLOADS — JSON array of DeployRequest (easyenclave/src/workload.rs:35-48)
//   SM_* — sm-specific env passed into each workload's env
//
// Two flavors:
//   sm_config: the one-per-customer sm VM — boots register, agent, chat, cloudflared
//   child_config: a plain easyenclave-stable VM — boots whatever the caller asked for

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub cmd: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(default)]
    pub tty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_release: Option<Value>,
    // dd-level fields (not in ee's DeployRequest; wrapped by dd-agent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_deploy: Option<Vec<String>>,
}

pub struct SmConfigArgs<'a> {
    pub sm_id: &'a str,
    pub cookie_key: &'a str,
    pub anthropic_api_key: &'a str,
    pub factory_url: &'a str,
    pub sm_hostname: &'a str,
    pub chat_hostname: &'a str,
}

/// Bake the ee-config for a new sm VM.
///
/// Boot order (encoded via `until`-loops in later workloads' cmds):
///   1. sm-register — writes /var/lib/easyenclave/run/registered
///   2. sm-chat     — ttyd on 127.0.0.1:7682 wrapping tmux + claude
///   3. sm-agent    — :8080, cookie-checks /chat/*, waits on /registered
///   4. cloudflared — reads tunnel token from /registered
pub fn sm_config(args: SmConfigArgs<'_>) -> Value {
    let sm_env = vec![
        format!("SM_ID={}", args.sm_id),
        format!("SM_COOKIE_KEY={}", args.cookie_key),
        format!("SM_FACTORY_URL={}", args.factory_url),
        format!("SM_HOSTNAME_SM={}", args.sm_hostname),
        format!("SM_HOSTNAME_CHAT={}", args.chat_hostname),
    ];

    let boot = vec![
        // 1. register (one-shot)
        json!({
            "app_name": "sm-register",
            "cmd": ["/usr/local/bin/smctl", "register"],
            "env": sm_env,
        }),
        // 2. chat (loopback ttyd + tmux + claude)
        json!({
            "app_name": "sm-chat",
            "cmd": [
                "/bin/sh", "-c",
                "until [ -f /var/lib/easyenclave/run/registered ]; do sleep 1; done; \
                 exec /usr/local/bin/ttyd -i 127.0.0.1 -W -p 7682 \
                   tmux new-session -A -s claude -c /var/lib/easyenclave/workspace \
                   /opt/claude-code/bin/claude"
            ],
            "tty": true,
            "env": [
                format!("ANTHROPIC_API_KEY={}", args.anthropic_api_key),
                "HOME=/var/lib/easyenclave/workspace".to_string(),
                "PATH=/usr/local/bin:/opt/claude-code/bin:/opt/gcloud/bin:/usr/bin:/bin".to_string(),
                format!("SM_ID={}", args.sm_id),
            ],
        }),
        // 3. agent (cookie-gated reverse proxy + /health)
        json!({
            "app_name": "sm-agent",
            "cmd": [
                "/bin/sh", "-c",
                "until [ -f /var/lib/easyenclave/run/registered ]; do sleep 1; done; \
                 exec /usr/local/bin/smctl agent"
            ],
            "env": sm_env,
            "expose": [
                { "hostname_label": "sm", "port": 8080 },
                { "hostname_label": "chat", "port": 8080 }
            ],
        }),
        // 4. cloudflared (ingress)
        json!({
            "app_name": "cloudflared",
            "cmd": [
                "/bin/sh", "-c",
                "until [ -f /var/lib/easyenclave/run/registered ]; do sleep 1; done; \
                 TOKEN=$(/bin/jq -r .tunnel_token /var/lib/easyenclave/run/registered); \
                 exec /usr/local/bin/cloudflared tunnel run --token $TOKEN"
            ],
        }),
    ];

    json!({
        "EE_BOOT_WORKLOADS": serde_json::to_string(&boot).expect("boot workloads serialize"),
    })
}

/// Bake the ee-config for a child workload VM (e.g. openclaw).
/// Caller supplies the full DeployRequest array — we just wrap it.
pub fn child_config(boot_workloads: &[Value]) -> Value {
    json!({
        "EE_BOOT_WORKLOADS": serde_json::to_string(boot_workloads)
            .expect("child boot workloads serialize"),
    })
}
