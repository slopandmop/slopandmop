# slopandmop

**Compose a pile of slop, slap it on a VM, mop up when you're done.**

A workload is _slop_. A running deployment is the pigs eating it. When
you're done, you _mop_ the pen. Farming.

This repo is a **catalog** of reference slop — plain
[easyenclave][ee] workload JSONs, no slopandmop-specific schema,
no magic. The one here is an end-to-end LLM stack: a static podman
tarball, a bootstrap workload that installs the wrapper, optional
NVIDIA driver insmod, an ollama serve container, and an openclaw
gateway in front of it.

## How to actually run this

This repo is a **template**. Two paths:

**One-click (coming via slopandmop.com):** visit [slopandmop.com](https://slopandmop.com), enter your agent's hostname, click "Deploy." The site authenticates against GitHub in your browser, forks this repo into your account, wires up `DD_AGENT_URL` + `DD_PAT`, and kicks off the deploy workflow. No YAML editing on your side.

**By hand, today:**

1. Click **"Use this template" → Create a new repository** on GitHub.
2. In your new repo, **Settings → Secrets and variables → Actions:**
    - Variable `DD_AGENT_URL` = `https://dd-production-agent-<id>.devopsdefender.com` (whatever your agent's hostname is)
    - Secret `DD_PAT` = a GitHub PAT owned by the agent's `DD_OWNER` org/user
3. **Actions tab → Deploy slop → Run workflow.** Pick a model (default `qwen2.5:7b`) and an ollama variant (`prod.json` for GPU agents, `preview.json` for CPU). The workflow POSTs each piece in order and prints a status summary.

Push to `main` of the forked repo also redeploys — edit the slop, push, the fork takes care of the rest.

Agents are owner-scoped: your PAT authorises you against any agent whose `DD_OWNER` is an org you belong to. Openclaw asks for `openclaw.<agent-hostname>` as its public hostname; dd's runtime ingress wires that up automatically once the workload posts (see [devopsdefender/dd#137][dd-137]).

### Bake + POST manually (no Actions)

If you just want to try the primitives:

```
export DD_PAT="$(gh auth token)"
curl -H "Authorization: Bearer $DD_PAT" \
     -H "Content-Type: application/json" \
     --data-binary "$(./bake.sh podman-static/workload.json)" \
     https://<agent-hostname>/deploy
# …repeat per workload in order:
#   nv (GPU only) → podman-static → podman-bootstrap
#   → ollama/workload.{preview,prod}.json
#   → MODEL=qwen2.5:7b ./bake.sh openclaw/workload.json.tmpl
```

## Layout

| path                                | role                                                                              |
|-------------------------------------|-----------------------------------------------------------------------------------|
| `podman-static/workload.json`       | fetch the [mgoltzsche/podman-static][ps] tarball into `/var/lib/easyenclave/bin`. |
| `podman-bootstrap/workload.json`    | stage binaries, install the `podman` wrapper + `containers.conf` + `policy.json`. |
| `nv/workload.json`                  | insmod the NVIDIA driver (GPU-agent only).                                        |
| `mount-models/workload.json`        | mount `/dev/vdc` at `/var/lib/easyenclave/ollama` for persistent model state.     |
| `ollama/workload.preview.json`      | CPU-only `ollama serve` via the podman wrapper.                                   |
| `ollama/workload.prod.json`         | GPU-enabled `ollama serve` with `--device=/dev/nvidia*` pass-through.             |
| `openclaw/workload.json.tmpl`       | openclaw gateway; `${MODEL}` is baked at deploy time.                             |
| `bake.sh`                           | render one `.json` or `.json.tmpl` → compact JSON (envsubst + jq).                |

## Ordering

EE spawns boot workloads concurrently; dependents self-sequence via
`until`-loops. `podman-bootstrap` waits for `podman-static`'s tarball;
`ollama`'s cmd waits for `/var/lib/easyenclave/bin/podman`; openclaw
waits for ollama's `/api/tags`. Cost: a few seconds of wasted polling
at boot. Benefit: zero dependency-graph code in the workload runner.

## Writing your own slop

Copy one of these dirs, rewrite `cmd` / `env`, POST the baked JSON to
an agent's `/deploy`. The schema is defined in
[`src/workload.rs`][workload] in easyenclave — anything EE's
`DeployRequest` deserializer accepts is valid slop.

If a workload binds an HTTP port and you want it reachable from the
public internet, add an `expose: {hostname_label, port}` field —
dd-agent forwards that to its control plane, which extends the
cloudflared tunnel's ingress at runtime.

## What's next for this repo

- [`easydollar`][edollar] — economic layer (compute credits, settlement). Returning as slop here.
- [`satsforcompute`][sfc] — BTC-for-compute marketplace. Returning as slop here.

[dd]: https://github.com/devopsdefender/dd
[dd-137]: https://github.com/devopsdefender/dd/pull/137
[ee]: https://github.com/easyenclave/easyenclave
[workload]: https://github.com/easyenclave/easyenclave/blob/main/src/workload.rs
[ps]: https://github.com/mgoltzsche/podman-static
[edollar]: https://github.com/easyenclave/easydollar
[sfc]: https://github.com/satsforcompute
