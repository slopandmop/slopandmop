// sm-provision-workload — CLI Claude Code inside sm invokes to provision a
// child workload VM. NOT a long-running process; returns once the child's
// primary_expose hostname is reachable.
//
//   smctl provision-workload --spec workloads/openclaw/workload.json.tmpl --model qwen2.5:7b
//     → reads the spec, subs ${MODEL}, bundles it into the full boot-workload
//       sequence (nv → mount-models → podman-static → podman-bootstrap → ollama → openclaw),
//       POSTs /sm/provision-workload to factory, polls, prints the public URL.

use anyhow::Result;
use std::path::PathBuf;

pub async fn run(spec: PathBuf, model: Option<String>, ttl_seconds: u64) -> Result<()> {
    tracing::info!(?spec, ?model, ttl_seconds, "provision-workload starting");
    // TODO:
    //  1. Read spec; `envsubst`-equivalent on ${MODEL}, ${OLLAMA_SPEC}.
    //  2. Assemble the child's full EE_BOOT_WORKLOADS array (for openclaw:
    //     nv → mount-models → podman-static → podman-bootstrap → ollama → openclaw).
    //  3. Fetch our own ITA token from configfs-tsm.
    //  4. POST /sm/provision-workload to factory with ITA token bearer.
    //  5. Print `{child_id, public_url}` as JSON on stdout.
    anyhow::bail!("provision_workload::run not yet implemented")
}

pub async fn teardown(child_id: String) -> Result<()> {
    tracing::info!(%child_id, "teardown-workload starting");
    // TODO: POST /sm/teardown-workload.
    anyhow::bail!("provision_workload::teardown not yet implemented")
}
