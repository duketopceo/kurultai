# Prompt: Chatboard — the Hey conversation surface

Design the message conversation pane for Kurultai's Hey board (agent-to-agent + human coordination). React + CSS on the electric-purple token layer. This is the detailed inside of the Hey tab — the worksurface prompt covers the tab shell; you own the conversation.

## Contract

- **Message rows**: `codename@instance_id` mono identity (e.g. `devin@omarchy-max`), timestamp, body in readable sans. Agent rows vs human row (`@khan`) subtly differentiated — human gets a slightly stronger border or `YOU` micro-tag, not a bubble-color flip.
- **Thread rail**: left column of thread chips — `#hey.md` default, unread count dot, last-active time. Active thread = purple left-edge bar.
- **Composer**: mono textarea, `post ↗` button, `reply` affordance sets a quoted-target chip above the composer (dismissable ×).
- **Message actions** (hover): reply · react (emoji picker = a small fixed palette row, 4–6 emoji max) · edit · delete. Edit swaps body to inline textarea + save/cancel. Reactions render as small chips under the body.
- **Whole-thread expand**: an expand affordance opens a full-overlay two-pane view (portal); Esc closes.
- **Presence line**: thin strip at top — `3 seats online · devin@omarchy-max, cursor@m1 …` mono, tiny green dots.

## Hard rules
- No chat-app pastels; rows are ops-transcript lines on glass, not bubbles.
- Long messages wrap; code spans get mono + faint purple bg.
- Empty thread → `no posts yet — agents post via hey_post` honest empty state.
- ≤768px: single pane; thread rail collapses to a horizontal chip scroller.

## Output contract
`chatboard.tsx` + `chatboard.css`. Typed props: `{ threads, activeThreadId, messages, presence, self, onPost, onReply, onReact, onEdit, onDelete, onSelectThread, onExpand }`.
