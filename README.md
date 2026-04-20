# slopandmop

**Compose a pile of slop, slap it on a VM, mop up when you're done.**

Each sm is a Claude Code process running inside a sealed Intel TDX VM, with
a markdown workspace as its state. The visitor hits [slopandmop.com][smcom],
the factory provisions an sm, and the visitor lands in a chat with their own
AI sysadmin that can provision more TDX VMs to run the reference slop
(openclaw, ollama, podman, …).

## Shape

Three long-lived pieces, orchestrated via [easyenclave][ee] + [dd][dd]:

1. **[slopandmop.com][smcom]** — static landing, one button.
2. **factory** (this repo, `SM_MODE=factory`) — dd-CP-shaped axum service on its
   own TDX VM. Holds `compute.admin` for our GCP project and the Anthropic API
   key. Provisions sm VMs, registers them, tears them down on a TTL.
3. **sm-\<id\>** — a per-customer TDX VM from GCE image family
   `slopandmop-stable`. Runs easyenclave as PID 1 with four boot workloads:
   `sm-register`, `sm-chat` (ttyd wrapping `tmux new-session -A -s claude
   claude`), `sm-agent` (cookie-gated reverse proxy → ttyd + /health), and
   `cloudflared`.

```
Visitor → slopandmop.com → POST factory/sm/provision
  Factory: bake ee-config, gcloud compute instances create, poll /health,
           mint cookie, 302 → chat.sm-<id>.slopandmop.com/?session=<cookie>
Visitor ↔ sm-agent ↔ ttyd ↔ tmux ↔ claude  (markdown workspace)
Claude: Bash → sm-provision-workload → factory → new child TDX VM → public URL
Watchdog: gcloud instances delete at teardown_at
```

Full design in `../.claude/plans/i-want-to-redo-sequential-tiger.md` (or in the
PR description for `feat/rewrite`).

## Layout

```
.
├── Cargo.toml
├── src/                               # single crate, multiple modes
│   ├── main.rs                        # dispatch on `smctl <mode>`
│   ├── config.rs                      # env: SM_MODE, SM_ID, SM_COOKIE_KEY, …
│   ├── cookie.rs                      # HS256 session cookie sign/verify
│   ├── ita.rs                         # verify Intel ITA tokens
│   ├── gcloud.rs                      # shell wrapper over `gcloud compute …`
│   ├── ee_config.rs                   # bake EE_BOOT_WORKLOADS for children
│   ├── factory.rs                     # axum: /sm/provision, /sm/register, /sm/provision-workload, /sm/teardown-workload
│   ├── watchdog.rs                    # 60 s cron: reap VMs past teardown_at
│   ├── agent.rs                       # sm-agent: cookie-check reverse proxy to ttyd; /health
│   ├── register.rs                    # sm-register: one-shot POST /sm/register with ITA token
│   └── provision_workload.rs          # sm-provision-workload: CLI sm uses to ask factory for a child VM
├── image/
│   ├── overlay/
│   │   └── opt/sm-seed/
│   │       ├── CLAUDE.md              # baked seed for workspace/CLAUDE.md
│   │       └── commands/              # /deploy-openclaw, /status, /teardown
│   └── targets/gcp-sm/profile.env     # fork of easyenclave's gcp target
├── workloads/                         # reference slop
│   ├── nv/workload.json
│   ├── mount-models/workload.json
│   ├── podman-static/workload.json
│   ├── podman-bootstrap/workload.json
│   ├── ollama/workload.{preview,prod}.json
│   └── openclaw/workload.json.tmpl
└── .github/workflows/
    ├── release.yml                    # cargo build smctl → GH release
    ├── image.yml                      # build+push slopandmop-stable GCE image
    └── deploy-factory.yml             # deploy factory to its TDX VM
```

## Modes

`smctl` is one binary that picks a mode at startup:

| mode                  | runs where              | does                                                                             |
|-----------------------|-------------------------|----------------------------------------------------------------------------------|
| `factory`             | factory TDX VM          | axum server on `:8080`, serves `/sm/*` endpoints + watchdog task.                |
| `agent`               | every sm TDX VM         | axum server on `:8080`, cookie-checks `/chat/*` → loopback ttyd; serves `/health`. |
| `register`            | every sm TDX VM (once)  | POSTs `/sm/register` to factory with ITA token, writes tunnel token to disk.     |
| `provision-workload`  | inside sm, called by Claude | reads a workload JSON, POSTs `/sm/provision-workload`, polls, prints URL.       |

## Reference slop

`workloads/` holds plain easyenclave `DeployRequest` JSONs ([schema][schema]).
Claude Code inside sm reads them, subs variables, passes them to
`sm-provision-workload`. Writing your own slop is committing another JSON
here.

## Not yet built

This repo was just rewritten on `feat/rewrite`. Everything under `src/` is
scaffolding — it compiles to stubs that print a mode banner and exit. Build
order against the plan:

- [ ] `factory.rs` + `cookie.rs` + `ita.rs` + `gcloud.rs` + `ee_config.rs`
- [ ] `agent.rs` — sm-side reverse proxy + /health
- [ ] `register.rs` + `provision_workload.rs`
- [ ] `image/` — forked easyenclave gcp target with sm overlay
- [ ] `.github/workflows/{release,image,deploy-factory}.yml`
- [ ] wire openclaw workload through the full path end-to-end

## Links

- [easyenclave][ee] — the TDX runtime
- [dd][dd] — the control plane / agent we reuse
- [slopandmop.com][smcom] — the landing page

[ee]: https://github.com/easyenclave/easyenclave
[dd]: https://github.com/devopsdefender/dd
[smcom]: https://slopandmop.com
[schema]: https://github.com/easyenclave/easyenclave/blob/main/src/workload.rs
