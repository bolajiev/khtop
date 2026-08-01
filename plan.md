# plan.md — khtop

Research and decision log for the KeeperHub Agents Onchain Hackathon submission.

## The hackathon

- **Host:** DoraHacks, "KeeperHub - Agents Onchain Hackathon"
- **Build phase:** July 27 – August 13, 2026
- **Submission deadline:** August 13, 2026, 12:00 UTC+2
- **Judging:** Aug 13–20. Winners announced Aug 20.
- **Prizes:** one open Grand Prize ranking across all submissions, plus a
  stackable $1,000 bounty (split two ways) for "Best Onboarding UX
  Improvement" — that bounty requires a merged PR into KeeperHub's own repo,
  so it's out of scope for us; we're building our own standalone project for
  the Grand Prize track.
- **Submission requirements:** GitHub link, a short demo video of the agent
  executing onchain through KeeperHub, and a link to the actual transaction.
  Incomplete submissions aren't judged.
- **Judging criteria (as stated):**
  1. Real onchain execution via KeeperHub — a working transaction, not a mockup
  2. Use of KeeperHub's surfaces — MCP server, CLI, x402, MPP, workflow
     builder, audit trail
  3. Reliability/observability — does the build understand failure modes
     (retries, gas handling, audit trail usage)
  4. Originality and real-world usefulness — would anyone actually run this
  5. Integration quality and developer experience

## What KeeperHub is

Execution and reliability layer for onchain AI agents, built by former
Sky/MakerDAO ops people. Agents decide *what* to do; KeeperHub guarantees the
transaction lands — gas estimation, retries, private/MEV-protected routing,
full audit trail. Non-custodial org wallet secured by Turnkey (hardware
enclaves). Supports Ethereum (gas-sponsored on mainnet), Base, Arbitrum,
Polygon, Sepolia.

Three ways in:
- **Visual workflow builder** (web) — trigger → actions → conditions
- **CLI (`kh`)** — full command surface, see below
- **REST API** — same surface as CLI, typed JSON
- **MCP server** (remote HTTP, `app.keeperhub.com/mcp`) — lets any MCP-aware
  agent (Claude Code, OpenCode, etc.) discover and call KeeperHub as tools
- **x402 / MPP** — pay-per-execution over HTTP for autonomous callers

Relevant CLI commands (from docs.keeperhub.com/cli):
```
kh auth login                    # browser auth, token in OS keyring
kh workflow list / get / create / run
kh run status / logs / cancel
kh execute contract-call         # fire one onchain action, no workflow needed
kh execute transfer
kh wallet balance / tokens / info
kh protocol list
```
`kh execute transfer` / `kh execute contract-call` are the fastest path to a
real, linkable transaction — no need to build a full workflow with
triggers/conditions just to satisfy the submission requirement.

## What already exists (landscape)

- **KeeperHub's own Claude Code plugin** already does a *chat-driven* version
  of "operate KeeperHub from your terminal" — skills like `workflow-builder`,
  `execution-monitor`, `template-browser` respond to natural-language prompts
  inside Claude Code. This is turn-based Q&A, tied to Claude Code specifically.
- **KeeperHub's own reference example** (in their marketing) is a Spark
  auto-compounder agent — "monitor my position, claim + compound when rewards
  > $50." Expect this shape to be the most crowded submission category.
- **KeeperHub's blog narrative** leans hard on security-incident framing
  (Resolv, Moonwell, Bybit) — expect a wave of "watchdog/security agent"
  submissions riding that same pitch.
- **Namera** (a BUIDL from a different DoraHacks hackathon) — an open-source
  scoped-permission wallet layer for agents on ZeroDev/account abstraction.
  Adjacent (solves key custody/authorization scope) but not overlapping —
  doesn't touch execution reliability or UX.
- No existing terminal *dashboard* (persistent, glanceable, multi-pane) for
  KeeperHub was found in searches — their tooling is web builder + CLI + REST
  + a conversational Claude Code plugin, nothing that shows live state on one
  screen without asking a question each time. Treated as a real gap, not
  confirmed absent (only checked their own repos and general search).

## The gap / the bet

KeeperHub gives you retries, gas handling, and an audit trail — but the only
ways to *see* any of that are the web app or asking their chat plugin a
question. There's no `htop`/`k9s`-style live view: workflows, run status,
wallet balance, and a streaming audit trail on one screen, always on.

That's the bet: build the ops view KeeperHub's own tooling doesn't have.
Directly on-thesis for judging criteria #2 (surface usage), #3
(reliability/observability), and #5 (integration quality/DX) — not just a
DeFi bot that happens to fire one transaction through KeeperHub.

## Decisions made

| Decision | Choice | Why |
|---|---|---|
| Build tool | OpenCode (not Claude Code) | No dependency — REST/CLI integration, framework-agnostic |
| Interface shape | Standalone terminal dashboard (TUI) | The gap that isn't covered by their own plugin |
| Language/stack | Rust + ratatui | Matches existing skillset, fast to ship a TUI solo |
| Data source | KeeperHub REST API directly | Typed JSON, no dependency on `kh` binary being installed, more control than parsing CLI text output |
| Chain for demo tx | Ethereum mainnet | KeeperHub sponsors gas there — removes cost blocker |
| Prize track | Grand Prize, own repo | UX bounty needs a merged PR into their repo — outside our control on this timeline |
| Name | **khtop** | Immediately legible to anyone who's used `htop`/`k9s`/`btop` — signals the category instantly, short enough to type during a live demo |

## Next steps (unblocking the build)

1. Pull exact request/response shapes from `docs.keeperhub.com/api/authentication`,
   `/api/workflows`, `/api/executions`, `/api/direct-execution`.
2. Spike: get an API key or `kh auth login`, fire one `kh execute transfer`
   on Sepolia manually to confirm the account/wallet/gas flow works before
   any UI is built.
3. Hand `khtop.md` to the build agent (OpenCode) once the spike confirms the
   integration works end to end.

See `khtop.md` for the build spec.
