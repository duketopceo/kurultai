import React from 'react';
import {Series, interpolate, useCurrentFrame} from 'remotion';
import {color, font, radius} from '../../../brands/kurultai/tokens';
import {StageFit} from '../../../core/Stage';
import {E1Hook} from './E1Hook';
import {E2Context} from './E2Context';
import {E3Notes} from './E3Notes';
import {E4Embed} from './E4Embed';
import {E5Mechanism} from './E5Mechanism';
import {E6End} from './E6End';

/**
 * Kurultai explainer — "how does an agent remember anything between sessions?"
 * Draft pacing (~80s); scene boundaries re-time to the user's VO timing.json.
 * Reference style: KodeKloud "How X Actually Works" — stage boxes, dashed
 * draw-on arrows, persistent prompt anchor bottom-left.
 */

// draft beats, frames @60 — replaced by VO timing when narration lands
const BEATS = {
  S1: [0, 780], // hook — yesterday's agent vs tomorrow's
  S2: [780, 1440], // context window wipes
  S3: [1440, 2100], // notes file: 0 matches
  S4: [2100, 3000], // embeddings — meaning space
  S5: [3000, 4200], // kurultai: ring, source+trust, propose→approve
  S6: [4200, 4800], // end card
};

/** Persistent anchor — the user prompt that follows the whole film. */
const Anchor: React.FC = () => {
  const f = useCurrentFrame();
  const o = interpolate(f, [30, 60], [0, 1], {extrapolateLeft: 'clamp'});
  // the query appears once the viewer meets the question (S3), stays after
  const showQuery = f >= 1440 && f < 4200;
  return (
    <div
      style={{
        position: 'absolute',
        left: 80,
        bottom: 36,
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        padding: '12px 22px',
        borderRadius: 999,
        border: `1.5px solid ${color.hairline}`,
        background: `${color.surfaceSunk}ee`,
        fontFamily: font.mono,
        fontSize: 17,
        color: color.ink2,
        opacity: o,
      }}
    >
      <span style={{color: color.accentSoft}}>agent ›</span>
      {showQuery ? 'what did we decide about tick batching?' : 'new session'}
      <span style={{opacity: f % 40 < 24 ? 1 : 0, color: color.ink}}>▌</span>
    </div>
  );
};

export const KurultaiExplainer: React.FC = () => (
  <StageFit>
    <Series>
      <Series.Sequence durationInFrames={BEATS.S1[1] - BEATS.S1[0]}>
        <E1Hook />
      </Series.Sequence>
      <Series.Sequence durationInFrames={BEATS.S2[1] - BEATS.S2[0]}>
        <E2Context />
      </Series.Sequence>
      <Series.Sequence durationInFrames={BEATS.S3[1] - BEATS.S3[0]}>
        <E3Notes />
      </Series.Sequence>
      <Series.Sequence durationInFrames={BEATS.S4[1] - BEATS.S4[0]}>
        <E4Embed />
      </Series.Sequence>
      <Series.Sequence durationInFrames={BEATS.S5[1] - BEATS.S5[0]}>
        <E5Mechanism />
      </Series.Sequence>
      <Series.Sequence durationInFrames={BEATS.S6[1] - BEATS.S6[0]}>
        <E6End />
      </Series.Sequence>
    </Series>
    <Anchor />
  </StageFit>
);

export const EXPLAINER_FRAMES = BEATS.S6[1];
