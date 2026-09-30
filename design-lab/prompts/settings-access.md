# Prompt: Settings + Access surfaces

Design the Settings panel and the human Access/login surface for the Kurultai dashboard. React + CSS on the electric-purple token layer.

## Settings (small — it sheds weight in the rebuild)

Contains only what exists for real:
- **Daemon**: version, online status, instance name (e.g. `Ulaanbaatar`), uptime if provided.
- **Access**: human login management — shows current access mode (open / token / cloudflare), a button to open the token/settings affordance.
- **Agent seats**: REAL data only — list comes from a `presence` prop (`codename@instance_id`, last-seen). Empty list → `no seats have posted presence`. NEVER a hardcoded agent list.
- Layout/tier/theme controls do NOT live here (they're in the top strip; there is no light theme).

## Access gate (HumanAccess)

A password-manager-friendly login card shown when the hosted daemon requires a human credential:
- Centered card on the dark canvas: brand mark, `KURULTAI` wordmark, one-line explanation.
- Single input — labeled `access token` — a REAL `<input type="password">`-style field with `autocomplete="current-password"` so 1Password/managers can save+fill. Reject paste of a whole CSV/key list (show inline error).
- `continue` button + subtle `or sign in with Cloudflare →` link when `cfAccess` prop is true.
- Error state: mono red line under the input. Loading: button spinner.
- No "LOCAL BRAIN" copy — hosted-safe wording: `this brain is private — enter your key`.

## Output contract
`settings-access.tsx` + `settings-access.css`. Props: `{ daemon, presence, mode, onLogin, cfAccess }`.
