import React from 'react';
import {interpolate, useCurrentFrame} from 'remotion';
import {MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 07 — End card (25–28s).
 * Brain shrinks to mark; command + URL. Held to the last frame.
 */
export const Scene07End: React.FC = () => {
  const frame = useCurrentFrame();
  const z = interpolate(frame, [0, 60], [4.6, 8.5], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const fade = interpolate(frame, [30, 70], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <BrainStage3D cameraZ={z}>
        <Brain3D ignite={1} edgeT={1} orbit={frame / 100} />
      </BrainStage3D>

      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'flex-end',
          paddingBottom: 130,
          gap: 28,
          opacity: fade,
        }}
      >
        <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 64, color: color.ink}}>
          kurultai <span style={{color: color.ink3, fontSize: 40}}>v0.7.0</span>
        </div>
        <div
          style={{
            padding: '14px 30px',
            borderRadius: radius.lg,
            background: color.surface,
            border: `1.5px solid ${color.hairline}`,
          }}
        >
          <MonoLine text="kurultai daemon --demo" size={28} ink={color.ink} delay={40} />
        </div>
        <MonoLine text="github.com/duketopceo/kurultai" size={24} ink={color.ink2} delay={60} />
      </div>
    </div>
  );
};
