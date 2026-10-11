import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {SynthCursor} from '../../../core/SynthCursor';
import {color} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 05 — Gate (16–20s).
 * A new edge hovers, pulsing pending. A cursor taps it; it locks green.
 * Physical metaphor for propose/decide. Beat: "you approve what it learns."
 */
export const Scene05Gate: React.FC = () => {
  const frame = useCurrentFrame();
  const locked = frame >= 120;
  const pulse = 0.5 + 0.5 * Math.sin(frame / 8);
  const orbit = frame / 170;

  // cursor path: drifts in from lower right to the pending edge midpoint
  const tapT = interpolate(frame, [40, 110], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.3, 0, 0.3, 1),
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <BrainStage3D cameraZ={4.4}>
        <Brain3D
          ignite={1}
          edgeT={1}
          orbit={orbit}
          pulse={pulse}
          pendingLocked={locked}
        />
      </BrainStage3D>

      <div style={{position: 'absolute', top: 90, left: 0, right: 0}}>
        <KineticType
          text={locked ? 'humans decide.' : 'agents propose.'}
          delay={4}
          size={58}
        />
      </div>

      <SynthCursor
        keys={[
          {at: 40, x: 1500, y: 850},
          {at: 110, x: 1090, y: 560},
          {at: 118, x: 1090, y: 560, click: true},
        ]}
      />

      <div style={{position: 'absolute', bottom: 110, left: 0, right: 0, textAlign: 'center'}}>
        <MonoLine
          text={locked ? 'edge approved — prop:9ca0a523' : 'promote_atom — pending'}
          size={24}
          ink={locked ? color.passed : color.caution}
          delay={20}
        />
      </div>
    </div>
  );
};
