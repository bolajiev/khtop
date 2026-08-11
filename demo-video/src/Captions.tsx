import {useMemo} from 'react';
import {useCurrentFrame, useVideoConfig} from 'remotion';
import words from './words.json';
import {C} from './theme';

type Word = {w: string; t: number; d: number};
type Sentence = {t: number; d: number; text: string};

const W: Word[] = words.words;
const S: Sentence[] = words.sentences;

// chunk words into sentences by time window
const CHUNKS: {words: Word[]; start: number; end: number}[] = (() => {
  const chunks: {words: Word[]; start: number; end: number}[] = [];
  for (const s of S) {
    const ws = W.filter((w) => w.t >= s.t - 0.02 && w.t < s.t + s.d);
    if (ws.length) chunks.push({words: ws, start: s.t, end: s.t + s.d});
  }
  return chunks;
})();

export const Captions: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const time = frame / fps;

  const active = useMemo(() => {
    return CHUNKS.find((c) => time >= c.start - 0.05 && time < c.end);
  }, [time]);

  if (!active) return null;

  let activeIdx = -1;
  for (let i = 0; i < active.words.length; i++) {
    const w = active.words[i];
    if (time >= w.t && time < w.t + w.d) {
      activeIdx = i;
      break;
    }
  }
  if (activeIdx < 0 && time >= active.end) return null;

  const fadeIn = Math.min(1, (time - active.start) / 0.18);
  const fadeOut = Math.min(1, (active.end - time) / 0.18);
  const opacity = Math.min(fadeIn, fadeOut) * 0.96;

  return (
    <div
      style={{
        position: 'absolute',
        bottom: 110,
        left: 0,
        right: 0,
        display: 'flex',
        justifyContent: 'center',
        pointerEvents: 'none',
      }}
    >
      <div
        style={{
          opacity,
          maxWidth: 1500,
          padding: '16px 34px',
          borderRadius: 14,
          background: 'rgba(2,6,23,0.62)',
          border: '1px solid rgba(30,46,71,0.9)',
          backdropFilter: 'blur(6px)',
          fontFamily: 'JetBrains Mono',
          fontSize: 34,
          lineHeight: 1.5,
          textAlign: 'center',
          color: C.faint,
        }}
      >
        {active.words.map((w, i) => {
          let color = C.faint;
          if (i < activeIdx) color = C.dim;
          if (i === activeIdx) color = C.white;
          if (i === activeIdx) {
            return (
              <span key={i}>
                <span style={{color: C.green, fontWeight: 700}}>{w.w}</span>{' '}
              </span>
            );
          }
          return (
            <span key={i}>
              <span style={{color}}>{w.w}</span>{' '}
            </span>
          );
        })}
      </div>
    </div>
  );
};
