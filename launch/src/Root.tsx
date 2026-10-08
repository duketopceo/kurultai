import React from 'react';
import {Composition} from 'remotion';
import {
  KurultaiLaunch,
  KurultaiLaunchProto,
  KurultaiLaunchSquare,
  KurultaiReadmeLoop,
} from './videos/kurultai/launch';
import {WebglProbe} from './videos/kurultai/launch/Probe';
import {
  KurultaiLaunchV3,
  KurultaiLaunchV3Square,
  KurultaiV3Loop,
} from './videos/kurultai/v3';
import {KurultaiExplainer, EXPLAINER_FRAMES} from './videos/kurultai/explainer';

export const Root: React.FC = () => {
  return (
    <>
      <Composition
        id="WebglProbe"
        component={WebglProbe}
        durationInFrames={60}
        fps={60}
        width={1920}
        height={1080}
      />
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
        durationInFrames={2520}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiLaunchSquare"
        component={KurultaiLaunchSquare}
        durationInFrames={2520}
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
      <Composition
        id="KurultaiLaunchV3"
        component={KurultaiLaunchV3}
        durationInFrames={1680}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiLaunchV3Square"
        component={KurultaiLaunchV3Square}
        durationInFrames={1680}
        fps={60}
        width={1080}
        height={1080}
      />
      <Composition
        id="KurultaiV3Loop"
        component={KurultaiV3Loop}
        durationInFrames={300}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiExplainer"
        component={KurultaiExplainer}
        durationInFrames={EXPLAINER_FRAMES}
        fps={60}
        width={1920}
        height={1080}
      />
      <Composition
        id="KurultaiExplainer916"
        component={KurultaiExplainer}
        durationInFrames={EXPLAINER_FRAMES}
        fps={60}
        width={1080}
        height={1920}
      />
    </>
  );
};
