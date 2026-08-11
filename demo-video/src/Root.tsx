import {Composition} from 'remotion';
import {DemoVideo} from './DemoVideo';

export const FPS = 30;
export const TOTAL_SECONDS = 58;
export const TOTAL_FRAMES = FPS * TOTAL_SECONDS;

export const RemotionRoot: React.FC = () => {
  return (
    <Composition
      id="Demo"
      component={DemoVideo}
      durationInFrames={TOTAL_FRAMES}
      fps={FPS}
      width={1920}
      height={1080}
    />
  );
};
