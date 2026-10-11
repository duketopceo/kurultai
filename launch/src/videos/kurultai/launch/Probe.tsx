import React from 'react';
import {Canvas} from '@react-three/fiber';
import {useCurrentFrame} from 'remotion';
import {color} from '../../../brands/kurultai/tokens';

const Glow: React.FC = () => {
  const frame = useCurrentFrame();
  const pulse = 1 + 0.15 * Math.sin(frame / 8);
  return (
    <mesh scale={pulse}>
      <icosahedronGeometry args={[1.4, 1]} />
      <meshStandardMaterial
        color={color.accentDeep}
        emissive={color.accent}
        emissiveIntensity={0.8}
        wireframe
      />
    </mesh>
  );
};

/** WebGL renderability probe — one pulsing wireframe orb, nothing else. */
export const WebglProbe: React.FC = () => (
  <div style={{position: 'absolute', inset: 0, background: color.canvas}}>
    <Canvas camera={{position: [0, 0, 5], fov: 50}}>
      <ambientLight intensity={0.4} />
      <pointLight position={[4, 4, 4]} intensity={2} color={color.accentSoft} />
      <Glow />
    </Canvas>
  </div>
);
