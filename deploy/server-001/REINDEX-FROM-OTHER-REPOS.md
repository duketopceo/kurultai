# Call from any duketopceo repo after deploy to reindex the Brain Repos mirror.
# Requires: org/repo secrets for a PAT that can dispatch to kurultai-private,
# OR copy the SSH reindex job into that repo's workflow.
#
# Example step (after successful deploy):
#
#   - name: Reindex Kurultai repos
#     env:
#       GH_TOKEN: ${{ secrets.KURULTAI_REINDEX_TOKEN }}
#     run: |
#       gh api repos/duketopceo/kurultai-private/dispatches \
#         -f event_type=kurultai-reindex \
#         -f client_payload[repo]="${{ github.repository }}" \
#         -f client_payload[sha]="${{ github.sha }}"
#
# On kurultai-private, enable workflow from:
#   deploy/server-001/github-deploy-workflow.yml.example
# as .github/workflows/deploy-server-001.yml with secrets:
#   KURULTAI_DEPLOY_HOST, KURULTAI_DEPLOY_SSH_KEY, KURULTAI_DEPLOY_USER
