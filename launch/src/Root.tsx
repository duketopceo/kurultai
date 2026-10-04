import React from 'react';
import {Composition} from 'remotion';
import {
  KurultaiLaunch,
  KurultaiLaunchProto,
  KurultaiLaunchSquare,
  KurultaiReadmeLoop,
} from './videos/kurultai/launch';

export const Root: React.FC = () => {
  return (
    <>
      <Composition
        id="KurultaiLaunchProto"
        component={KurultaiLaunchProto}
        durationInFrames={420}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiLaunch"
        component={KurultaiLaunch}
        durationInFrames={2160}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiLaunchSquare"
        component={KurultaiLaunchSquare}
        durationInFrames={2160}
        fps={60}
        width={1080}
        height={1080}
      />
      <Composition
        id="KurultaiReadmeLoop"
        component={KurultaiReadmeLoop}
        durationInFrames={450}
        fps={60}
        width={1920}
        height={1080}
      />
    </>
  );
};
