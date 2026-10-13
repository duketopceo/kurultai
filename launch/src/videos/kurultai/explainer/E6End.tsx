import React from 'react';
import {interpolate, spring, useCurrentFrame} from 'remotion';
import {color, font} from '../../../brands/kurultai/tokens';
import SynapseMark from '../../../brands/kurultai/BrainMark';

/** S6: end card — mark, claim, repo. */
export const E6End: React.FC = () => {
  const f = useCurrentFrame();
  const mark = spring({frame: f, fps: 60, config: {damping: 200}});
  const o = interpolate(f, [30, 60], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});

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
        gap: 26,
      }}
    >
      <div style={{transform: `scale(${mark})`, opacity: mark}}>
        <SynapseMark size={110} />
      </div>
      <div style={{fontFamily: font.grotesk, fontSize: 72, fontWeight: 700, color: color.ink, opacity: o}}>
        kurultai
      </div>
      <div style={{fontFamily: font.grotesk, fontSize: 30, color: color.ink2, opacity: o}}>
        shared, cited, governed memory for agent fleets
      </div>
      <div
        style={{
          marginTop: 14,
          fontFamily: font.mono,
          fontSize: 22,
          color: color.accentSoft,
          opacity: o,
        }}
      >
        github.com/duketopceo/kurultai
      </div>
      <div style={{fontFamily: font.mono, fontSize: 16, color: color.ink3, opacity: o}}>
        MIT · kurultai daemon · kurultai mcp
      </div>
    </div>
  );
};
