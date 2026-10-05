import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType, MonoLine} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

/**
 * Scene 07 — onboarding split.
 * Left: run it local — `daemon` + `kurultai mcp` in each agent config.
 * Right: run it on your server — `docker compose up`, agents connect
 * over tailscale / https. The demo literally runs this way today.
 * Beat: "Run it here. Or there."
 */
export const Scene07Onboard: React.FC = () => {
  const frame = useCurrentFrame();

  const leftIn = interpolate(frame, [14, 40], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const rightIn = interpolate(frame, [50, 76], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });
  const linkIn = interpolate(frame, [95, 125], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0, 1),
  });

  const card = (
    title: string,
    lines: string[],
    t: number,
    left: number,
  ): React.ReactNode => (
    <div
      style={{
        position: 'absolute',
        left,
        top: 290,
        width: 700,
        padding: 32,
        borderRadius: radius.lg,
        background: color.surface,
        border: `1.5px solid ${color.hairline}`,
        opacity: t,
        transform: `translateY(${(1 - t) * 50}px)`,
      }}
    >
      <div style={{fontFamily: font.grotesk, fontWeight: 700, fontSize: 32, color: color.ink}}>
        {title}
      </div>
      <div style={{marginTop: 20, display: 'flex', flexDirection: 'column', gap: 12}}>
        {lines.map((l) => (
          <MonoLine key={l} text={l} size={24} ink={color.ink2} delay={0} />
        ))}
      </div>
    </div>
  );

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 72, left: 0, right: 0}}>
        <KineticType text="Run it here. Or there." delay={4} size={64} />
      </div>

      {card('local', [
        '$ kurultai daemon',
        '# one line per agent config:',
        '"command": "kurultai mcp"',
        '# web ui optional',
      ], leftIn, 160)}

      {card('your server', [
        '$ docker compose up -d',
        '# agents connect over',
        'tailscale · https://host',
        '# this is the live demo',
      ], rightIn, 1060)}

      {/* connective tissue: both end at the same brain */}
      <div
        style={{
          position: 'absolute',
          left: 860,
          top: 700,
          width: 200,
          textAlign: 'center',
          opacity: linkIn,
          fontFamily: font.mono,
          fontSize: 22,
          color: color.accentSoft,
        }}
      >
        same store.
        <br />
        same tools.
      </div>
    </div>
  );
};
