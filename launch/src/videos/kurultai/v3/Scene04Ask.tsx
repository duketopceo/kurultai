import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 04 — Ask (12–16s).
 * Plain-language question types; one neuron lights; the answer card
 * slides out carrying its source. The whole product in one shot.
 * Beat: "ask where it came from."
 */
export const Scene04Ask: React.FC = () => {
  const frame = useCurrentFrame();
  const askT = interpolate(frame, [10, 55], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const litT = interpolate(frame, [70, 92], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const cardT = interpolate(frame, [95, 125], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <BrainStage3D cameraZ={4.4}>
        <Brain3D
          ignite={1}
          edgeT={1}
          orbit={frame / 170}
          activeNode={litT > 0.5 ? 2 : -1}
        />
      </BrainStage3D>

      {/* question types */}
      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 130,
          width: 800,
          padding: '18px 28px',
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.hairline}`,
          opacity: askT > 0 ? 1 : 0,
        }}
      >
        <MonoLine
          text='kurultai ask "how does tick batching work?"'
          size={26}
          ink={color.ink}
          delay={12}
        />
      </div>

      {/* answer card with provenance */}
      <div
        style={{
          position: 'absolute',
          left: 620,
          top: 640,
          width: 680,
          padding: 26,
          borderRadius: radius.lg,
          background: color.raised,
          border: `1.5px solid ${color.accent}55`,
          opacity: cardT,
          transform: `translateY(${(1 - cardT) * 60}px)`,
        }}
      >
        <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 28, color: color.ink}}>
          ADR-004: batch ingest ticks at 250ms
        </div>
        <div style={{display: 'flex', gap: 10, marginTop: 14, alignItems: 'center'}}>
          <span
            style={{
              fontFamily: font.mono,
              fontSize: 17,
              padding: '4px 14px',
              borderRadius: 999,
              color: color.accentSoft,
              border: `1px solid ${color.accentDeep}`,
            }}
          >
            source: 12-meridian-adr-tick-batching.md
          </span>
          <span
            style={{
              fontFamily: font.mono,
              fontSize: 16,
              padding: '4px 12px',
              borderRadius: 999,
              color: color.passed,
              border: `1px solid ${color.passed}66`,
            }}
          >
            trusted
          </span>
        </div>
      </div>

      <div style={{position: 'absolute', bottom: 90, left: 0, right: 0, textAlign: 'center'}}>
        <MonoLine text="every answer carries its source" size={24} ink={color.ink2} delay={140} />
      </div>
    </div>
  );
};
