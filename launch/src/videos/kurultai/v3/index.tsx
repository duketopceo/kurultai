import React from 'react';
import {Sequence} from 'remotion';
import {StageFit} from '../../../core/Stage';
import {Scene01Dots} from './Scene01Dots';
import {Scene02Snap} from './Scene02Snap';
import {Scene03Live} from './Scene03Live';
import {Scene04Ask} from './Scene04Ask';
import {Scene05Gate} from './Scene05Gate';
import {Scene06ZoomOut} from './Scene06ZoomOut';
import {Scene07End} from './Scene07End';

/**
 * v3 — the 3D-brain cut. 7 beats / 1680 frames = 28s @60fps.
 * Nearly wordless: the brain carries the product, type beats are captions.
 *
 *   0–3s   SiloDots   five dots, five private memory orbits
 *   3–7s   Snap       "kurultai mcp" → dots converge → brain ignites
 *   7–12s  LiveBrain  edges fire, supersede dims a node, slow orbit
 *   12–16s Ask        question → neuron lights → answer + source card
 *   16–20s Gate       pending edge pulses → cursor tap → locks green
 *   20–25s ZoomOut    dolly back: same brain behind laptop + server
 *   25–28s End        brain → mark; daemon --demo + repo URL
 */
export const KurultaiLaunchV3: React.FC = () => (
  <StageFit>
    <Sequence durationInFrames={180}>
      <Scene01Dots />
    </Sequence>
    <Sequence from={180} durationInFrames={240}>
      <Scene02Snap />
    </Sequence>
    <Sequence from={420} durationInFrames={300}>
      <Scene03Live />
    </Sequence>
    <Sequence from={720} durationInFrames={240}>
      <Scene04Ask />
    </Sequence>
    <Sequence from={960} durationInFrames={240}>
      <Scene05Gate />
    </Sequence>
    <Sequence from={1200} durationInFrames={300}>
      <Scene06ZoomOut />
    </Sequence>
    <Sequence from={1500} durationInFrames={180}>
      <Scene07End />
    </Sequence>
  </StageFit>
);

/** Square 1:1 for X feed — the inner StageFit scales, never crops. */
export const KurultaiLaunchV3Square: React.FC = () => <KurultaiLaunchV3 />;

/** README loop — ignite + live brain, 5s, loops seamlessly. */
export const KurultaiV3Loop: React.FC = () => (
  <StageFit>
    <Sequence durationInFrames={300}>
      <Scene03Live />
    </Sequence>
  </StageFit>
);
