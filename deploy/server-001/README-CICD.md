# Deploy from git (self-hosted runner on server-001)

Merging to `main` runs `.github/workflows/deploy-server-001.yml` on a runner
that lives on server-001. It fast-forwards `/home/khan/kurultai` to the pushed
SHA (never forces), runs `redeploy.sh` (build, cold backup, swap, health gate,
auto-rollback), then reindexes. A failed health gate fails the job. No
`pull_request` trigger exists, so PR code never reaches the runner. Docs-only
pushes are skipped; `workflow_dispatch` and `repository_dispatch`
(`kurultai-reindex`, reindex only) are also accepted.

## One-time owner steps

1. GitHub: Settings > Environments > New `production` (add yourself as required
   reviewer if you want a manual gate; restrict deployment branches to `main`).
2. Get a registration token (expires in about 1 hour):
   `timeout 60 gh api -X POST repos/duketopceo/kurultai/actions/runners/registration-token --jq .token`
3. On server-001 as `khan` (arm64 or x64 build from the repo's Settings >
   Actions > Runners > New self-hosted runner page):
   ```
   mkdir -p ~/actions-runner && cd ~/actions-runner
   # download + extract the runner tarball shown on that page
   ./config.sh --url https://github.com/duketopceo/kurultai --token <TOKEN> \
     --labels self-hosted,server-001,kurultai-deploy --name server-001-kurultai --unattended
   sudo ./svc.sh install khan && sudo ./svc.sh start
   ```
4. Ensure `khan` is in the `docker` group and `/home/khan/kurultai` is clean on
   `main` with a server-local `.env` (the runner never touches secrets).
5. Keep the repo private-fork-PR safe: Settings > Actions > General > require
   approval for outside collaborators. The old `KURULTAI_DEPLOY_*` secrets are
   no longer used and can be deleted.
