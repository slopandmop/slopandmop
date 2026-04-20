# /status — show what's running

Report on all child deployments this sm has provisioned.

## Steps

1. List `deployments/*.md`.
2. For each: read its front-matter. If `status: running` or `deploying` and
   `teardown_at` is not in the past, print one line:
   ```
   <child_id>  <workload>  <hostname>  ttl=<Xh Ym>  status=<status>
   ```
3. For any `status: deploying` entry, re-check by calling
   `sm-provision-workload --status --child-id <id>` and update the markdown.
4. End with a one-line summary: `<n> running, <m> torn-down, your sm expires
   in <t>`.

Don't hit the factory for `torn-down` entries — they're final.
