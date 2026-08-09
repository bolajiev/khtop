# khtop

A persistent, glanceable terminal dashboard for [KeeperHub](https://keeperhub.com) —
the `htop`/`k9s`-style view of what your workflows, executions, and wallet are
actually doing. Live, always-on, from your terminal.

Built for the [KeeperHub Agents Onchain Hackathon](https://dorahacks.io) (Grand Prize track).

## Why

KeeperHub gives you retries, gas handling, and a full audit trail — but the
only ways to *see* any of that are the web app or asking a chat plugin a
question. khtop puts it on one screen:

- **Runs** — unified live feed of workflow executions and direct executions
  (status, source, workflow, gas used, transaction, time)
- **Workflows** — your workflow list with trigger type and last update
- **Audit tail** — per-step logs for the selected run: trigger → simulation →
  submitted tx → gas used → outcome, streamed live while the run is in flight
- **Wallet / gas** — daily spend cap gauge, org gas spend, success rate, chains,
  sponsorship status

## Proof of execution

Real transactions executed through KeeperHub's Direct Execution API from this
project (Sepolia, org wallet `0xb80640aab9a0b123b2d56a3ad0cb1d11b129e00b`):

| Execution | Transaction | Link |
|---|---|---|
| `khtop --transfer` spike (0.0001 ETH, self-transfer) | `0x8eb57aa6...2ca9b4` | https://sepolia.etherscan.io/tx/0x8eb57aa69edce99aae83ce970c36ff59b01ffb23985d0f73b05794ef9c2ca9b4 |
| direct API broadcast (0.0001 ETH, self-transfer) | `0x4665cde5...95a1e7` | https://sepolia.etherscan.io/tx/0x4665cde5c87c37d697e87a94a454f2d05bd614e696aa6a76de89c4dc0e95a1e7 |

Both follow the documented safe first-write sequence: `simulate: true` dry-run
first (gas estimate, revert check), then broadcast with an `Idempotency-Key`,
then status polling for the on-chain proof.

## KeeperHub surfaces used

- **REST API** (`app.keeperhub.com/api`) — workflows, executions/logs,
  analytics (runs, summary, spend-cap), chains
- **Direct execution API** — transfers with `simulate` dry-run, `Idempotency-Key`,
  and `X-RateLimit-*` / polling-hint header handling
- **Audit trail** — per-step logs (gas used, tx hash, explorer link, errors)
- **Gas handling** — spend-cap tracking, gas-used from run logs, sponsorship notes

## Setup

```sh
cargo build --release

cp .env.example .env          # add your KH_API_KEY (kh_...)
# KeeperHub: app.keeperhub.com -> Settings -> API Keys -> Organisation tab
```

### Demo transfer config (for the `t` action)

```sh
KH_DEMO_CHAIN_ID=11155111     # Sepolia by default; 1 for Ethereum mainnet
KH_DEMO_RECIPIENT=0x...       # where the demo transfer sends ETH
KH_DEMO_AMOUNT=0.0001         # amount in ETH
```

## Usage

```sh
khtop                          # dashboard
khtop --once                   # validate key + dump dashboard data as JSON
khtop --simulate-transfer      # dry-run the demo transfer (no tx broadcast)
khtop --transfer               # simulate, then broadcast the demo transfer
```

### Keys

| Key | Action |
|---|---|
| `j`/`k`, `↑`/`↓` | move selection |
| `Enter` | load audit logs for selected run |
| `Tab` | switch pane (runs → workflows → logs → gas) |
| `r` | trigger a run of the selected workflow |
| `t` | direct transfer: enter amount → simulate (dry-run) → confirm → broadcast |
| `/` | filter runs (id, status, source, workflow) |
| `PgUp`/`PgDn` | scroll the audit tail |
| `?` | help |
| `q` / `Ctrl-C` | quit |

The transfer flow follows KeeperHub's documented safe first-write sequence:
`simulate: true` first (catches reverts, bad ABI, insufficient balances), and
only then broadcast with an `Idempotency-Key` so an interrupted client can
retry safely.

## Failure-mode awareness

- Rate limits: the client reads `X-RateLimit-Remaining` and displays remaining
  budget; polling is paced at ~5s to stay well inside the documented
  100 req/min
- Errors: surfaces the machine-readable `error` code, `detail`, and
  `request_id` (the correlation id KeeperHub asks you to quote)
- Retry/backoff for transient failures (5xx, rate limits) per the documented
  strategy; a failed refresh degrades the header state without killing the UI
- Reverted simulations are never broadcast

## License

MIT
