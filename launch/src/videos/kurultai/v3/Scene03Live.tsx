import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {color} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 03 — LiveBrain (7–12s).
 * Edges fire in order (zero-LLM extraction), camera orbits slow.
 * The superseded node dims mid-scene. Beat: "it learns on its own."
 */
export const Scene03Live: React.FC = () => {
  const frame = useCurrentFrame();
  const edgeT = interpolate(frame, [10, 150], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const supT = interpolate(frame, [160, 200], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const camZ = interpolate(frame, [0, 240], [4.2, 5.4], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <BrainStage3D cameraZ={camZ}>
        <Brain3D ignite={1} edgeT={edgeT} supersedeT={supT} orbit={frame / 160} />
      </BrainStage3D>

      <div style={{position: 'absolute', top: 90, left: 0, right: 0}}>
        <KineticType text="it wires itself — no LLM in the loop." delay={4} size={54} />
      </div>
      <div style={{position: 'absolute', bottom: 120, left: 0, right: 0, textAlign: 'center'}}>
        <MonoLine text="edges extracted at index · atoms supersede · --as-of rewinds" size={24} ink={color.ink2} delay={120} />
      </div>
    </div>
  );
};
