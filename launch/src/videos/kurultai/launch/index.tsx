import React from 'react';
import {Series} from 'remotion';
import {StageFit} from '../../../core/Stage';
import {Scene01Silos} from './Scene01Silos';
import {Scene02Wire} from './Scene02Wire';
import {Scene01Hook} from './Scene01Hook';
import {Scene02Fleet} from './Scene02Fleet';
import {Scene03Atom} from './Scene03Atom';
import {Scene04Search} from './Scene04Search';
import {Scene05SelfWire} from './Scene05SelfWire';
import {Scene05Hey} from './Scene05Hey';
import {Scene06Ontology} from './Scene06Ontology';
import {Scene07Onboard} from './Scene07Onboard';
import {Scene08EndCard} from './Scene08EndCard';

/**
 * Prototype composition — v1 hook + fleet scene (~7s).
 * Kept for reference; v2 uses Silos → Wire.
 */
export const KurultaiLaunchProto: React.FC = () => (
  <StageFit>
    <Series>
      <Series.Sequence durationInFrames={150}>
        <Scene01Hook />
      </Series.Sequence>
      <Series.Sequence durationInFrames={270}>
        <Scene02Fleet />
      </Series.Sequence>
    </Series>
  </StageFit>
);

/**
 * Full launch film v2 — problem → wiring → value → onboard. ~42s @60fps.
 * Storyboard: src/videos/kurultai/launch/storyboard.md
 */
export const KurultaiLaunch: React.FC = () => (
  <StageFit>
    <Series>
      <Series.Sequence durationInFrames={240}>
        <Scene01Silos />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene02Wire />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene03Atom />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene04Search />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene05SelfWire />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene05Hey />
      </Series.Sequence>
      <Series.Sequence durationInFrames={300}>
        <Scene06Ontology />
      </Series.Sequence>
      <Series.Sequence durationInFrames={240}>
        <Scene07Onboard />
      </Series.Sequence>
      <Series.Sequence durationInFrames={240}>
        <Scene08EndCard />
      </Series.Sequence>
    </Series>
  </StageFit>
);

/** Square 1:1 for X feed — StageFit letterboxes the 1920×1080 stage. */
export const KurultaiLaunchSquare: React.FC = () => (
  <StageFit>
    <KurultaiLaunch />
  </StageFit>
);

/** README loop — silent synapse + tagline, 7.5s. */
export const KurultaiReadmeLoop: React.FC = () => (
  <StageFit>
    <Series>
      <Series.Sequence durationInFrames={450}>
        <Scene01Hook />
      </Series.Sequence>
    </Series>
  </StageFit>
);
