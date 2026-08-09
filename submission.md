# khtop — DoraHacks submission kit

Hackathon: KeeperHub - Agents Onchain Hackathon
Deadline: 2026-08-13 10:00 UTC. Submit at https://dorahacks.io/hackathon/agents-onchain (button: "Submit BUIDL").

## Checklist

- [ ] GitHub repo: https://github.com/bolajiev/khtop (public, README, CI, LICENSE)
- [ ] Demo video (see options below)
- [ ] Transaction link: https://sepolia.etherscan.io/tx/0x8620e157459b0d7c85a5c7ca6f235176c030d968d9e8e9bc0ebef5746ce3eb10
      (broadcast from inside the TUI during the demo recording; two more in README)
- [ ] Submit the BUIDL (register as hacker first — wallet sign-in)

## Video options

1. (Recommended) Record a short screen video on your machine (OBS / Loom / phone):
   run `khtop`, follow the shot list below, upload to YouTube/Loom, paste the URL.
2. Asciinema: `asciinema upload demo.cast` (repo root) -> returns a URL with a
   player. Embeddable, but a screen video with the Etherscan page visible is
   stronger.

Shot list (under 2 minutes): dashboard boot -> Enter on a run (audit tail) ->
`t` -> amount -> Enter (simulate) -> Enter (broadcast) -> toast + the run
appearing in the feed -> open the sepolia.etherscan.io link in the browser ->
`?` help overlay. All keys shown on screen.

## BUIDL page copy

Title: **khtop — terminal dashboard for KeeperHub**

Tagline: The htop/k9s-style live view of your KeeperHub workflows, executions,
audit trail, and wallet — always on, in your terminal.

Description:

khtop is a persistent, glanceable terminal dashboard for KeeperHub. KeeperHub
gives agents retries, gas handling, and a full audit trail — but the only ways
to see any of that are the web app or asking a chat plugin a question. khtop
puts it on one screen: a live runs feed (workflow executions + direct
executions with status, gas used, and transactions), your workflow list with
trigger types, a per-step audit tail (trigger -> simulation -> submitted tx ->
gas used -> outcome, streamed live), and a wallet/gas panel with daily
spend-cap and sponsorship state.

It executes onchain too: press `t` and a transfer goes through KeeperHub's
Direct Execution API using their documented safe first-write sequence —
`simulate: true` dry-run first (gas estimate, revert check, decoded revert
reason), then broadcast with an Idempotency-Key, then status polling with the
on-chain proof. Failures are surfaced, not hidden: a reverted simulation is
never broadcast, a failed run shows its error and trace in the audit tail, and
rate-limit budget is tracked live from the API headers. It degrades gracefully
when the API key's scope excludes analytics endpoints (falls back to
per-workflow execution history).

KeeperHub surfaces used: REST API (workflows, executions/logs, chains),
Direct Execution API (simulate, Idempotency-Key, X-Poll-Interval-Hint,
X-RateLimit-*), audit trail, gas handling.

Real transactions on Sepolia (links in the repo README), plus an asciinema
recording of a transfer executed from inside the TUI.

Tags: KeeperHub, terminal, TUI, observability, automation, web3, DeFi, Rust

## Notes for judges

- No chat interface, no workflow authoring, no MCP wrapper — those already
  exist. khtop is the ops view KeeperHub's own tooling lacks.
- Reliability is the point: failure surfacing, scope degradation, retry-aware
  polling within documented rate limits (100 req/min), and the safe
  simulate-before-broadcast sequence.
- The UX bounty requires a merged PR into KeeperHub's repo — out of scope for
  this submission (Grand Prize track).
