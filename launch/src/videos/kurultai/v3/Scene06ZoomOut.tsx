import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {Brain3D, BrainStage3D} from '../../../core/Brain3D';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * v3 Scene 06 — ZoomOut (20–25s).
 * Camera dollies back; the same brain sits behind a laptop card AND a
 * server card. Beat: "run it here. or there."
 */
export const Scene06ZoomOut: React.FC = () => {
  const frame = useCurrentFrame();
  const zoom = interpolate(frame, [10, 90], [4.0, 7.5], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.4, 0, 0.2, 1),
  });
  const cardT = (d: number) =>
    interpolate(frame, [d, d + 26], [0, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: Easing.bezier(0.2, 0, 0, 1),
    });

  const Chip: React.FC<{x: number; title: string; sub: string; t: number}> = ({x, title, sub, t}) => (
    <div
      style={{
        position: 'absolute',
        left: x,
        top: 700,
        width: 470,
        padding: '20px 28px',
        borderRadius: radius.lg,
        background: color.surface,
        border: `1.5px solid ${color.hairline}`,
        opacity: t,
        transform: `translateY(${(1 - t) * 50}px)`,
      }}
    >
      <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 30, color: color.ink}}>{title}</div>
      <div style={{fontFamily: font.mono, fontSize: 18, color: color.ink3, marginTop: 6, whiteSpace: 'nowrap'}}>{sub}</div>
    </div>
  );

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <BrainStage3D cameraZ={zoom}>
        <Brain3D ignite={1} edgeT={1} orbit={frame / 140} />
      </BrainStage3D>

      <div style={{position: 'absolute', top: 90, left: 0, right: 0}}>
        <KineticType text="run it here. or there." delay={30} size={58} />
      </div>

      <Chip x={230} title="local" sub="kurultai daemon · kurultai mcp" t={cardT(80)} />
      <Chip x={1220} title="your server" sub="docker compose · tailscale · https" t={cardT(100)} />
    </div>
  );
};
