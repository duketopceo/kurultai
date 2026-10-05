import React from 'react';
import {Easing, interpolate, useCurrentFrame} from 'remotion';
import {KineticType} from '../../../core/KineticType';
import {color, font, radius} from '../../../brands/kurultai/tokens';

const AGENTS = ['cursor', 'claude', 'codex', 'hermes', 'devin'];

/**
 * Scene 01 v2 — the honest problem.
 * Five agent chips, each with its OWN walled-off memory bubble.
 * No connections between them — the silo is the shot.
 * Beat: "Every agent has its own memory. None of them share it."
 */
export const Scene01Silos: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
      <div style={{position: 'absolute', top: 72, left: 0, right: 0}}>
        <KineticType
          text="Every agent has its own memory."
          delay={6}
          size={64}
        />
      </div>

      {/* five silos — each an agent + its private memory bubble */}
      {AGENTS.map((a, i) => {
        const cx = 384 * i + 192; // five centers: 192..1728
        const deal = interpolate(frame, [30 + i * 10, 52 + i * 10], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.bezier(0.2, 0, 0, 1),
        });
        const bubbleGrow = interpolate(frame, [80 + i * 8, 105 + i * 8], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
          easing: Easing.bezier(0.2, 0, 0, 1),
        });
        return (
          <React.Fragment key={a}>
            {/* agent chip */}
            <div
              style={{
                position: 'absolute',
                left: cx - 90,
                top: 380,
                width: 180,
                padding: '18px 0',
                borderRadius: radius.lg,
                background: color.raised,
                border: `1.5px solid ${color.hairline}`,
                textAlign: 'center',
                fontFamily: font.mono,
                fontSize: 24,
                color: color.ink,
                opacity: deal,
                transform: `translateY(${(1 - deal) * 60}px)`,
              }}
            >
              {a}
            </div>
            {/* private memory bubble — same muted color for all: nothing shared */}
            <div
              style={{
                position: 'absolute',
                left: cx - 70,
                top: 500,
                width: 140,
                height: 140,
                borderRadius: '50%',
                background: color.surface,
                border: `1.5px dashed ${color.ink3}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                opacity: bubbleGrow,
                transform: `scale(${0.2 + bubbleGrow * 0.8})`,
              }}
            >
              <span style={{fontFamily: font.mono, fontSize: 17, color: color.ink3}}>
                memory
              </span>
            </div>
            {/* wall between silos */}
            {i < AGENTS.length - 1 && (
              <div
                style={{
                  position: 'absolute',
                  left: cx + 192,
                  top: 350,
                  width: 2,
                  height: 320,
                  background: `repeating-linear-gradient(${color.hairline} 0 10px, transparent 10px 20px)`,
                  opacity: bubbleGrow,
                }}
              />
            )}
          </React.Fragment>
        );
      })}

      <div style={{position: 'absolute', bottom: 170, left: 0, right: 0}}>
        <KineticType
          text="None of them share it."
          delay={140}
          size={56}
          ink={color.ink2}
        />
      </div>
    </div>
  );
};
