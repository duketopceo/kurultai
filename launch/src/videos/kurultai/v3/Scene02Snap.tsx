import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 02 — Snap (3–7s).
 * `"kurultai mcp"` flashes once; the five dots SNAP into the brain;
 * it ignites — neurons pop, camera settles. Beat: "one brain."
 */
export const Scene02Snap: React.FC = () => {
  const frame = useCurrentFrame();

  const typeIn = interpolate(frame, [8, 40], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const snap = interpolate(frame, [55, 95], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.5, 0, 0.1, 1),
  });
  const ignite = interpolate(frame, [70, 130], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const dotFade = 1 - snap;

  // dots converge: silo x → center
  const centers = [340, 650, 960, 1270, 1580];

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      {/* brain ignites under the dots */}
      <BrainStage3D cameraZ={4.6 - snap * 0.7}>
        <Brain3D ignite={ignite} edgeT={0} orbit={frame / 220} />
      </BrainStage3D>

      {/* dots fly inward then yield to the 3D neurons */}
      {centers.map((cx, i) => {
        const x = cx + (960 - cx) * snap;
        const y = 500 + (540 - 500) * snap;
        return (
          <div
            key={i}
            style={{
              position: 'absolute',
              left: x - 13,
              top: y - 13,
              width: 26,
              height: 26,
              borderRadius: '50%',
              background: color.accent,
              boxShadow: `0 0 26px ${color.accent}`,
              opacity: dotFade,
            }}
          />
        );
      })}

      {/* the one-line config flash */}
      <div
        style={{
          position: 'absolute',
          left: 560,
          top: 180,
          padding: '14px 26px',
          borderRadius: radius.lg,
          background: color.surface,
          border: `1.5px solid ${color.hairline}`,
          opacity: typeIn * (1 - snap),
        }}
      >
        <MonoLine text='"command": "kurultai mcp"' size={26} ink={color.ink} delay={10} />
      </div>

      <div style={{position: 'absolute', bottom: 150, left: 0, right: 0}}>
        <KineticType text="one brain." delay={100} size={64} />
      </div>
    </div>
  );
};
