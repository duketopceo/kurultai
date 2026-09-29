# Call from any duketopceo repo after deploy to reindex the Brain Repos mirror.
# Requires: org/repo secrets for a PAT that can dispatch to kurultai,
# OR copy the SSH reindex job into that repo's workflow.
#
# Example step (after successful deploy):
#
#   - name: Reindex Kurultai repos
#     env:
#       GH_TOKEN: ${{ secrets.KURULTAI_REINDEX_TOKEN }}
#     run: |
#       gh api repos/duketopceo/kurultai/dispatches \
#         -f event_type=kurultai-reindex \
#         -f client_payload[repo]="${{ github.repository }}" \
#         -f client_payload[sha]="${{ github.sha }}"
#
# On kurultai, the listener ships as
#   .github/workflows/deploy-server-001.yml (repository_dispatch:
#   kurultai-reindex) with secrets:
#   KURULTAI_DEPLOY_HOST, KURULTAI_DEPLOY_SSH_KEY, KURULTAI_DEPLOY_USER
