# khtop.md

Build spec for **khtop** — a terminal dashboard for KeeperHub. Read this
before writing code. If something here conflicts with what you find in
KeeperHub's actual docs/API once you're building, the docs win — flag the
conflict and stop rather than guessing.

## What this is

khtop is a persistent, glanceable terminal UI (`htop`/`k9s`/`btop`-style) for
KeeperHub. KeeperHub already has a web builder, a CLI, and a chat-driven
Claude Code plugin — none of those give you a live, always-on view of what
your workflows and executions are actually doing. khtop is that view.

**It is not:** another chat interface, another workflow-authoring tool, or a
wrapper around their MCP server. Don't build any of those — they already
exist.

## Before writing code

### 1. Confirm the API surface, don't assume it
Fetch and read, in this order, before touching Rust:
- `docs.keeperhub.com/api/authentication`
- `docs.keeperhub.com/api/workflows`
- `docs.keeperhub.com/api/executions`
- `docs.keeperhub.com/api/direct-execution`
- `docs.keeperhub.com/api/organizations` and `/api-keys`

Record actual field names, pagination behavior, and error shapes
(`docs.keeperhub.com/api/errors`) before writing the client. Don't invent a
schema and fix it later — every wrong assumption here costs a rebuild of the
data layer.

### 2. Prove the integration before building UI
Get an API key (`app.keeperhub.com` → Settings → API Keys) or run
`kh auth login` locally. Fire one real transaction —
`kh execute transfer` or the REST equivalent — on **Sepolia** first (free,
no real funds at risk). Confirm you can see it land and pull its status/logs
back via the API. Only once that loop works end to end, move to mainnet
Ethereum for the final demo transaction (gas-sponsored by KeeperHub).

Do not start on the TUI layout until this loop is proven. A beautiful
dashboard showing nothing real is worse than an ugly one showing a real
transaction.

### 3. Stay inside scope
The smallest version that satisfies every judging criterion:
- Shows real workflows/runs pulled from the live API (not mocked data)
- Can trigger at least one action from inside the TUI (a workflow run or a
  direct execution) — the demo needs an interaction, not just a read-only view
- Streams run logs / status changes live, not on manual refresh only
- Surfaces failure states clearly — a failed run, a retry, gas info — since
  "does it understand failure modes" is graded explicitly
- Shows wallet balance / gas state somewhere on screen

Anything beyond that (multi-org switching, workflow authoring inside the
TUI, template browsing, notification config) is a later phase. Don't build
it for the hackathon deadline unless the core loop is done with time to
spare.

## Stack

- **Language:** Rust
- **TUI framework:** ratatui
- **HTTP client:** reqwest (or ureq if a lighter dependency tree matters) —
  talk to the KeeperHub REST API directly, don't shell out to the `kh`
  binary as the primary path. (A CLI fallback is fine as a secondary
  input method if it's cheap, but the REST client is the source of truth.)
- **Async runtime:** tokio, for polling/streaming without blocking the
  render loop
- **Auth storage:** read `KH_API_KEY` from env or config file; don't
  reimplement the OAuth browser flow — that's what the API key path is for

## Architecture sketch

Three panes, minimum:
1. **Workflows/runs list** — id, status, last run time, trigger type
2. **Live log/audit tail** — streams `run logs` for the selected run:
   trigger → simulation → submitted tx → gas used → outcome → timestamp
3. **Wallet/gas panel** — balance, chain, recent gas cost, sponsorship status

Polling interval should be short enough to feel "live" (a few seconds) but
respect KeeperHub's rate limits — check `docs.keeperhub.com/api` for any
stated limits before hardcoding an interval.

Keybinding pattern: match what `k9s`/`htop` users already expect —
number/tab to switch panes, `r` to trigger a run, `q` to quit, `/` to filter.
Don't invent an unfamiliar scheme; the point is zero learning curve for
anyone who already lives in a terminal.

## What "done" looks like for submission

- [ ] Source on GitHub, public, with a README explaining setup (API key,
      install, run)
- [ ] A real transaction executed through KeeperHub, linkable (Etherscan or
      equivalent), triggered from inside khtop
- [ ] A short demo video showing: khtop running, a workflow/run visible
      live, a transaction triggered from the TUI, the audit trail updating
      in response
- [ ] README explicitly states which KeeperHub surfaces are used (REST API,
      audit trail, gas handling) — judging criteria call this out directly,
      make it easy for judges to check the box

## Guardrails

- Never commit an API key or wallet credential. `.env` in `.gitignore` from
  commit one.
- Don't fake or mock a transaction for the demo — judging explicitly
  penalizes "a polished demo that never touches a chain."
- If the Sepolia spike (see above) fails or KeeperHub's actual API doesn't
  match this spec, stop and report back rather than building further on a
  wrong assumption — same rule as the research phase.
