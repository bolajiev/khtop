import {AbsoluteFill, Audio, Img, Sequence, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig} from 'remotion';
import {loadFont as loadInter} from '@remotion/google-fonts/Inter';
import {loadFont as loadMono} from '@remotion/google-fonts/JetBrainsMono';
import {Captions} from './Captions';
import {C, DOT_GRID, GLOW_GREEN, GLOW_INDIGO} from './theme';

const Inter = loadInter().fontFamily;
const Mono = loadMono().fontFamily;

// scene timeline (seconds)
const S = {title: 0, problem: 4.5, dash: 11.5, audit: 22, transfer: 31, tx: 42, outro: 50, end: 58};

const f = (sec: number) => Math.round(sec * 30);

const ease = (frame: number, from: number, to: number) =>
  interpolate(frame, [from, to], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

const Fade: React.FC<{frame: number; delay: number; dur?: number; children: React.ReactNode; style?: React.CSSProperties}> = ({
  frame, delay, dur = 0.4, children, style,
}) => {
  const v = interpolate(frame, [delay * 30, (delay + dur) * 30], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  return <div style={{opacity: v, transform: `translateY(${(1 - v) * 24}px)`, ...style}}>{children}</div>;
};

const Background: React.FC = () => (
  <AbsoluteFill style={{background: C.bg, backgroundImage: `${GLOW_INDIGO}, ${GLOW_GREEN}, ${DOT_GRID.backgroundImage}`, backgroundSize: 'auto, auto, 34px 34px'}} />
);

const ProgressBar: React.FC = () => {
  const frame = useCurrentFrame();
  const {durationInFrames} = useVideoConfig();
  const p = frame / durationInFrames;
  return (
    <div style={{position: 'absolute', bottom: 0, left: 0, right: 0, height: 4, background: 'rgba(30,46,71,0.6)'}}>
      <div style={{height: '100%', width: `${p * 100}%`, background: `linear-gradient(90deg, ${C.indigo}, ${C.green})`}} />
    </div>
  );
};

const TerminalCursor: React.FC<{frame: number}> = ({frame}) => {
  const blink = Math.floor(frame / 18) % 2 === 0;
  return (
    <span style={{display: 'inline-block', width: 26, height: 74, marginLeft: 14, background: blink ? C.green : 'transparent', verticalAlign: 'text-bottom'}} />
  );
};

const Shot: React.FC<{src: string; frame: number; delay: number; zoom?: number}> = ({src, frame, delay, zoom = 1.05}) => {
  const enter = spring({frame: frame - delay * 30, fps: 30, config: {damping: 22, stiffness: 140, mass: 0.9}});
  const ken = ease(frame, delay * 30, delay * 30 + 240);
  return (
    <div style={{opacity: enter, transform: `translateY(${(1 - enter) * 60}px) scale(${1 + (zoom - 1) * ken})`, display: 'flex', justifyContent: 'center'}}>
      <div style={{position: 'relative', border: `1px solid ${C.border}`, borderRadius: 14, overflow: 'hidden', boxShadow: `0 40px 120px rgba(2,6,23,0.9), 0 0 0 1px rgba(76,72,255,0.15), 0 0 90px rgba(76,72,255,0.10)`}}>
        <Img src={staticFile(src)} style={{width: 1480, display: 'block'}} />
      </div>
    </div>
  );
};

const Callout: React.FC<{frame: number; delay: number; color: string; children: React.ReactNode; style?: React.CSSProperties}> = ({
  frame, delay, color, children, style,
}) => {
  const v = interpolate(frame, [delay * 30, delay * 30 + 14], [0, 1], {extrapolateLeft: 'clamp'});
  return (
    <div style={{display: 'flex', alignItems: 'center', gap: 12, opacity: v, transform: `translateX(${(1 - v) * -18}px)`, ...style}}>
      <div style={{width: 4, height: 22, background: color, borderRadius: 2}} />
      <span style={{fontFamily: Mono, fontSize: 24, color: C.white}}>{children}</span>
    </div>
  );
};

const KeyChip: React.FC<{label: string}> = ({label}) => (
  <span style={{fontFamily: Mono, fontSize: 26, fontWeight: 700, color: C.green, border: `1.5px solid ${C.green}`, borderRadius: 8, padding: '2px 12px', margin: '0 4px'}}>{label}</span>
);

const SceneTitle: React.FC<{frame: number; delay: number; children: React.ReactNode; sub?: string}> = ({frame, delay, children, sub}) => (
  <Fade frame={frame} delay={delay}>
    <div style={{fontFamily: Inter, fontWeight: 800, fontSize: 56, color: C.white}}>{children}</div>
    {sub ? <div style={{fontFamily: Inter, fontWeight: 500, fontSize: 26, color: C.dim, marginTop: 12}}>{sub}</div> : null}
  </Fade>
);

export const DemoVideo: React.FC = () => {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill>
      <Background />
      <Audio src={staticFile('vo.mp3')} />

      {/* ---- 1. TITLE ---- */}
      <Sequence from={f(S.title)} durationInFrames={f(S.problem - S.title) + 15}>
        <AbsoluteFill style={{justifyContent: 'center', alignItems: 'center'}}>
          <Fade frame={frame} delay={0.15}>
            <div style={{fontFamily: Mono, fontWeight: 700, fontSize: 150, color: C.white, letterSpacing: 2}}>
              khtop<TerminalCursor frame={frame} />
            </div>
          </Fade>
          <Fade frame={frame} delay={0.75}>
            <div style={{fontFamily: Inter, fontSize: 32, fontWeight: 600, letterSpacing: 12, color: C.dim, marginTop: 18}}>
              TERMINAL DASHBOARD FOR KEEPERHUB
            </div>
          </Fade>
          <Fade frame={frame} delay={1.35}>
            <div style={{marginTop: 46, display: 'flex', alignItems: 'center', gap: 14, fontFamily: Mono, fontSize: 26, color: C.dim}}>
              <span style={{width: 10, height: 10, borderRadius: 5, background: C.green, boxShadow: `0 0 14px ${C.green}`}} />
              EXECUTION LAYER FOR ONCHAIN AGENTS
              <span style={{width: 10, height: 10, borderRadius: 5, background: C.indigo, boxShadow: `0 0 14px ${C.indigo}`}} />
            </div>
          </Fade>
          <Fade frame={frame} delay={2.1}>
            <div style={{marginTop: 90, display: 'flex', gap: 18, fontFamily: Mono, fontSize: 22, color: C.faint}}>
              <span>workflows</span><span style={{color: C.border}}>·</span>
              <span>executions</span><span style={{color: C.border}}>·</span>
              <span>audit trail</span><span style={{color: C.border}}>·</span>
              <span>gas</span>
            </div>
          </Fade>
        </AbsoluteFill>
      </Sequence>

      {/* ---- 2. PROBLEM ---- */}
      <Sequence from={f(S.problem)} durationInFrames={f(S.dash - S.problem) + 15}>
        <AbsoluteFill style={{padding: 70, flexDirection: 'column'}}>
          <SceneTitle frame={frame} delay={0.2}>
            Your agents run on KeeperHub.
            <br />Where's the live view?
          </SceneTitle>
          <Fade frame={frame} delay={1.1}>
            <div style={{fontFamily: Inter, fontSize: 28, color: C.dim, marginTop: 20, lineHeight: 1.6}}>
              Retries. Gas handling. Audit trails — all handled for you.
              <br />
              But watching them means the web app… or asking a chat plugin a question.
            </div>
          </Fade>
          <div style={{flex: 1}} />
          <Shot src="shot_dashboard.png" frame={frame} delay={2.0} zoom={1.06} />
        </AbsoluteFill>
      </Sequence>

      {/* ---- 3. DASHBOARD ---- */}
      <Sequence from={f(S.dash)} durationInFrames={f(S.audit - S.dash) + 15}>
        <AbsoluteFill style={{padding: 60, flexDirection: 'column'}}>
          <div style={{display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: 26}}>
            <SceneTitle frame={frame} delay={0.2}>
              Everything, on one screen
            </SceneTitle>
            <Fade frame={frame} delay={1.6}>
              <div style={{fontFamily: Mono, fontSize: 22, color: C.cyan}}>◉ LIVE · refreshes every 5s</div>
            </Fade>
          </div>
          <Shot src="shot_dashboard.png" frame={frame} delay={0.5} zoom={1.09} />
          <div style={{marginTop: 30, display: 'flex', gap: 40, flexWrap: 'wrap'}}>
            <Callout frame={frame} delay={2.2} color={C.green}>runs: status · source · gas · tx</Callout>
            <Callout frame={frame} delay={2.7} color={C.indigo}>workflows: trigger · last run</Callout>
            <Callout frame={frame} delay={3.2} color={C.cyan}>wallet &amp; spend cap, live</Callout>
          </div>
        </AbsoluteFill>
      </Sequence>

      {/* ---- 4. AUDIT ---- */}
      <Sequence from={f(S.audit)} durationInFrames={f(S.transfer - S.audit) + 15}>
        <AbsoluteFill style={{padding: 60, flexDirection: 'column'}}>
          <SceneTitle frame={frame} delay={0.2}>
            Every step, audited
          </SceneTitle>
          <Fade frame={frame} delay={0.9}>
            <div style={{fontFamily: Inter, fontSize: 27, color: C.dim, marginTop: 14}}>
              trigger → simulation → submitted tx → gas used → outcome
            </div>
          </Fade>
          <div style={{flex: 1}} />
          <Shot src="shot_audit.png" frame={frame} delay={1.4} zoom={1.08} />
          <div style={{marginTop: 26, display: 'flex', gap: 40}}>
            <Callout frame={frame} delay={3.4} color={C.red}>a failed run shows its error, right there</Callout>
            <Callout frame={frame} delay={3.9} color={C.yellow}>gas per step · tx hash · explorer link</Callout>
          </div>
        </AbsoluteFill>
      </Sequence>

      {/* ---- 5. TRANSFER ---- */}
      <Sequence from={f(S.transfer)} durationInFrames={f(S.tx - S.transfer) + 15}>
        <AbsoluteFill style={{padding: 60, flexDirection: 'column'}}>
          <SceneTitle frame={frame} delay={0.2}>
            Executes onchain — safely
          </SceneTitle>
          <Fade frame={frame} delay={0.8}>
            <div style={{marginTop: 18, fontFamily: Mono, fontSize: 24, color: C.dim, display: 'flex', alignItems: 'center', gap: 8}}>
              press <KeyChip label="t" /> enter amount <KeyChip label="0.0001" /> <KeyChip label="Enter" /> — simulate first, no broadcast
            </div>
          </Fade>
          <div style={{flex: 1}} />
          <div style={{display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 36}}>
            <Fade frame={frame} delay={1.5}>
              <Img src={staticFile('shot_transfer.png')} style={{width: 700, border: `1px solid ${C.border}`, borderRadius: 12}} />
            </Fade>
            <div style={{fontFamily: Mono, fontSize: 40, color: C.faint}}>→</div>
            <Fade frame={frame} delay={2.4}>
              <Img src={staticFile('shot_confirm.png')} style={{width: 700, border: `1px solid ${C.border}`, borderRadius: 12}} />
            </Fade>
          </div>
          <div style={{marginTop: 28, display: 'flex', gap: 40}}>
            <Callout frame={frame} delay={3.6} color={C.green}>reverts caught before broadcast</Callout>
            <Callout frame={frame} delay={4.1} color={C.indigo}>Idempotency-Key · gas handled by KeeperHub</Callout>
          </div>
        </AbsoluteFill>
      </Sequence>

      {/* ---- 6. TX ---- */}
      <Sequence from={f(S.tx)} durationInFrames={f(S.outro - S.tx) + 15}>
        <AbsoluteFill style={{padding: 60, flexDirection: 'column', justifyContent: 'center', alignItems: 'center'}}>
          <Fade frame={frame} delay={0.2}>
            <div style={{fontFamily: Mono, fontSize: 22, color: C.green, letterSpacing: 4, display: 'flex', alignItems: 'center', gap: 10}}>
              <span style={{width: 12, height: 12, borderRadius: 6, background: C.green, boxShadow: `0 0 16px ${C.green}`, animation: 'none'}} />
              EXECUTION COMPLETED · SEPOLIA
            </div>
          </Fade>
          <Fade frame={frame} delay={0.9}>
            <div style={{marginTop: 40, fontFamily: Mono, fontSize: 44, color: C.white, border: `1px solid ${C.border}`, borderRadius: 16, padding: '26px 44px', background: 'rgba(10,14,22,0.8)', boxShadow: `0 0 90px rgba(0,255,79,0.10)`}}>
              0x8620e157…ce3eb10
            </div>
          </Fade>
          <Fade frame={frame} delay={1.7}>
            <div style={{marginTop: 26, fontFamily: Mono, fontSize: 24, color: C.cyan}}>
              sepolia.etherscan.io/tx/0x8620e157…
            </div>
          </Fade>
          <div style={{marginTop: 56, display: 'flex', gap: 30}}>
            <Callout frame={frame} delay={2.6} color={C.green}>0.0001 ETH</Callout>
            <Callout frame={frame} delay={3.1} color={C.yellow}>gas sponsored</Callout>
            <Callout frame={frame} delay={3.6} color={C.indigo}>real · linkable · onchain</Callout>
          </div>
        </AbsoluteFill>
      </Sequence>

      {/* ---- 7. OUTRO ---- */}
      <Sequence from={f(S.outro)} durationInFrames={f(S.end - S.outro) + 15}>
        <AbsoluteFill style={{justifyContent: 'center', alignItems: 'center'}}>
          <Fade frame={frame} delay={0.2}>
            <div style={{fontFamily: Mono, fontWeight: 700, fontSize: 110, color: C.white}}>
              khtop<span style={{color: C.green}}>_</span>
            </div>
          </Fade>
          <Fade frame={frame} delay={0.9}>
            <div style={{marginTop: 20, fontFamily: Inter, fontSize: 30, color: C.dim, letterSpacing: 3}}>
              THE OPS VIEW YOUR AGENTS DESERVE
            </div>
          </Fade>
          <Fade frame={frame} delay={1.6}>
            <div style={{marginTop: 34, fontFamily: Mono, fontSize: 26, color: C.cyan}}>github.com/bolajiev/khtop</div>
          </Fade>
          <Fade frame={frame} delay={2.3}>
            <div style={{marginTop: 60, display: 'flex', gap: 20, fontFamily: Mono, fontSize: 22, color: C.dim}}>
              {['SEPOLIA-PROVEN', 'GAS-HANDLED', 'AUDIT-READY'].map((t) => (
                <span key={t} style={{border: `1px solid ${C.border}`, borderRadius: 999, padding: '8px 22px'}}>{t}</span>
              ))}
            </div>
          </Fade>
          <Fade frame={frame} delay={3.0}>
            <div style={{marginTop: 30, fontFamily: Inter, fontSize: 20, color: C.faint}}>
              KeeperHub Agents Onchain Hackathon
            </div>
          </Fade>
        </AbsoluteFill>
      </Sequence>

      <Captions />
      <ProgressBar />
    </AbsoluteFill>
  );
};
