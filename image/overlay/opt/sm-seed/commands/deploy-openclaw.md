# /deploy-openclaw — provision an openclaw gateway

Spins up a fresh TDX VM running the full ollama + openclaw stack.

Arguments (optional, ask the user if not provided):

- `model` — ollama model tag. Default `qwen2.5:7b`. For CPU-only: `qwen2.5:0.5b`.
- `ollama_variant` — `prod` (GPU pass-through) or `preview` (CPU). Default `prod`.
- `ttl_hours` — how long before auto-teardown. Default 2.

## Steps

1. If `ollama_variant=preview`, warn the user GPU pass-through is off and large
   models will be slow.
2. Run:
   ```bash
   sm-provision-workload \
     --spec workloads/openclaw/workload.json.tmpl \
     --model "$model" \
     --ttl-seconds $((ttl_hours * 3600))
   ```
3. Parse the `{child_id, public_url}` output.
4. Write `deployments/<child_id>.md` with front-matter (workload=openclaw,
   model, teardown_at, status=deploying) and initial notes.
5. Poll `sm-provision-workload --status --child-id <child_id>` until
   status is `running` or `failed` (give up after 5 minutes).
6. Update the markdown file's status, and reply to the user:
   > openclaw is up at <public_url>. Model: <model>. Auto-mop in <ttl_hours> hours.

If any step fails, tell the user the specific error and point them at
`sm-teardown-workload --child-id <id>` to clean up.
