# khtop — demo video

58-second product demo for [khtop](https://github.com/bolajiev/khtop), built
with [Remotion](https://www.remotion.dev/): dark fintech style, voiceover,
karaoke captions, real TUI screenshots, and a live onchain transaction.

Rendered output: `out/khtop-demo.mp4` (1920×1080, 30 fps, 58 s) — generated locally, not committed to the repo.

## Rebuild

```sh
npm install
npm run render          # -> out/khtop-demo.mp4
```

Requirements: Node 18+, network access on first render (Remotion downloads its
headless browser; Google Fonts are fetched at render time).

## Structure

```
src/
  index.ts        entry, registerRoot
  Root.tsx        composition: 1920x1080, 30 fps, 58 s
  DemoVideo.tsx   the whole film — 7 scenes, motion, audio
  Captions.tsx    karaoke captions driven by words.json timings
  theme.ts        brand palette (KeeperHub colors) and background treatments
  words.json      voiceover word timings (from edge-tts)
public/
  shot_*.png      real TUI screenshots (captured via tmux, ANSI -> PNG)
  vo.mp3          voiceover (edge-tts)
```

## Scene timeline (seconds)

| # | Scene | Content |
|---|---|---|
| 1 | 0–4.5 | Title: `khtop_` with blinking cursor, KeeperHub tagline |
| 2 | 4.5–11.5 | Problem: "where's the live view?" + dashboard screenshot |
| 3 | 11.5–22 | Dashboard callouts: runs, workflows, wallet |
| 4 | 22–31 | Audit trail: per-step logs, failed run with error |
| 5 | 31–42 | Transfer flow: `t` → amount → simulate → confirm |
| 6 | 42–50 | Real Sepolia transaction card |
| 7 | 50–58 | Outro: repo, hackathon, feature chips |

## Regenerating assets

**Screenshots** — capture the live TUI in tmux, then convert ANSI to PNG:

```sh
tmux new-session -d -s shot 'khtop'   # resize window, set status off
tmux capture-pane -t shot -pe > frame.ansi
python3 ../scripts/ansi2png.py frame.ansi frame.png
```

**Voiceover** — regenerate with a new script and word timings:

```sh
python3 -m pip install edge-tts
# see scripts/vo_script.txt for the narration; words.json is derived from
# edge-tts SentenceBoundary events with per-word durations distributed by
# character length
```

## Customization

- Voice: `voice="en-US-AriaNeural"` and `rate` in the generation step
- Timing: scene boundaries are the `S` constant at the top of `DemoVideo.tsx`
- Colors: `theme.ts` (KeeperHub brand: `#4C48FF` on `#020617`, `#00FF4F`)
- Captions: sentence/word chunking logic in `Captions.tsx`

## License

MIT (see repo root).
