# /teardown — tear down a specific child VM

Usage: `/teardown <child_id>` — or `/teardown <workload>` to tear down by
workload name (e.g. `openclaw`) if there's only one match.

## Steps

1. Resolve `<arg>` to a `child_id`:
   - If it looks like a child_id (short uuid), use directly.
   - Else grep front-matter in `deployments/*.md` for `workload: <arg>` and
     `status: running`. If exactly one match, use that id. If multiple, ask
     the user which.
2. Run `sm-teardown-workload --child-id <id>`.
3. On success, update the deployment's markdown: set `status: torn-down`
   and append a note with today's timestamp.
4. Reply: `torn down <child_id> (<workload>).`

If the user says "tear down everything", iterate over all `status: running`
deployments.
