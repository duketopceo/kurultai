// Argus reviewer configuration — code-review-only consumer.
// Kurultai has no per-PR web target (CLI + axum daemon on ephemeral ports),
// so the workflow runs the action with `run: 'false'`; no `target` is set here.
export default {
  // Diff review model via OpenRouter (BYOK).
  code_model: 'deepseek/deepseek-v4.1-flash',
  // Per-PR spend cap — Argus dogfood reviews run ~$0.05 at ~100k tokens.
  codeReviewBudgetUsd: 0.5,
  // decisionModel defaults to the pinned Jev slug — secrets adjudication is on.
  review: {
    maxComments: 20,
    severityGate: 'bug',
    secretsThreshold: 0.3,
  },
}
