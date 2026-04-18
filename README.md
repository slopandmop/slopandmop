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

The deployment UX — how a human clicks a button or pushes a branch
and the slop lands on their fleet — is in-flight over in
[`devopsdefender/dd`][dd]. Until that lands, slop here can be baked
and POSTed manually:

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

Agents are owner-scoped, so your GitHub PAT authorises you to deploy
onto any agent whose `DD_OWNER` is an org you belong to. Openclaw asks
for `openclaw.<agent-hostname>` as its public hostname; dd's runtime
ingress pipeline wires that up automatically once the workload posts
(see [devopsdefender/dd#137][dd-137]).

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
