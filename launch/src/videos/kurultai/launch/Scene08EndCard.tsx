import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {SynapseMark} from '../../../brands/kurultai/BrainMark';
import {MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * Scene 08 — End card (32–36s).
 * Mark re-draws; install + repo URL. Held ≥2s for a meaningful last frame.
 */
export const Scene08EndCard: React.FC = () => {
  const frame = useCurrentFrame();
  const cardIn = interpolate(frame, [8, 30], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        background: color.canvas,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 44,
      }}
    >
      <SynapseMark size={150} delay={0} tint={color.accent} />
      <div
        style={{
          fontFamily: font.grotesk,
          fontWeight: 700,
          fontSize: 68,
          color: color.ink,
          letterSpacing: '-0.02em',
          opacity: cardIn,
        }}
      >
        kurultai <span style={{color: color.ink3, fontSize: 44}}>v0.6.0</span>
      </div>
      <div
        style={{
          padding: '18px 34px',
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.hairline}`,
          opacity: cardIn,
        }}
      >
        <MonoLine text="kurultai daemon --demo" size={30} ink={color.ink} delay={30} />
      </div>
      <MonoLine text="github.com/duketopceo/kurultai" size={26} ink={color.ink2} delay={50} />
    </div>
  );
};
