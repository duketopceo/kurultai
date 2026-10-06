import React from 'react';
import * as THREE from 'three';
import {Canvas} from '@react-three/fiber';
import {color} from '../brands/kurultai/tokens';

/**
 * Brain3D — the kurultai synapse as a real 3D object.
 * Pure function of props; scenes drive it via useCurrentFrame math.
 *
 * - nodes: fixed positions (deterministic — no per-frame randomness)
 * - edges fire in order via `edgeT` (0..1 global progress)
 * - `activeNode` lights one neuron (search hit)
 * - `pendingEdge` pulses a dashed-looking edge; `pendingLocked` seals it gold
 */

const NODES: Array<[number, number, number]> = [
  [-1.6, 0.5, 0.2], // adr-004 (supersedable)
  [-0.9, 1.0, -0.4], // adr-005
  [0.2, 0.8, 0.5], // meridian
  [-0.4, -0.5, 0.6], // runbook-14
  [1.3, -0.3, -0.2], // exp-e13
  [0.7, -0.9, 0.3],
  [-1.1, -0.2, -0.6],
  [0.9, 0.4, 0.8],
  [1.7, 0.6, -0.5],
];

// [from, to, kind]
const EDGES: Array<[number, number, 'sup' | 'norm' | 'pend']> = [
  [0, 1, 'sup'],
  [1, 2, 'norm'],
  [2, 3, 'norm'],
  [2, 4, 'pend'],
  [3, 5, 'norm'],
  [6, 0, 'norm'],
  [7, 2, 'norm'],
  [8, 4, 'norm'],
];

const V = (n: number) => new THREE.Vector3(...NODES[n]);

const Edge: React.FC<{
  a: number;
  b: number;
  grow: number;
  kind: 'sup' | 'norm' | 'pend';
  locked: boolean;
  pulse: number;
}> = ({a, b, grow, kind, locked, pulse}) => {
  if (grow <= 0.001) return null;
  const va = V(a);
  const vb = V(b);
  const mid = va.clone().lerp(vb, grow / 2);
  const len = va.distanceTo(vb) * grow;
  const dir = vb.clone().sub(va).normalize();
  const quat = new THREE.Quaternion().setFromUnitVectors(
    new THREE.Vector3(0, 1, 0),
    dir,
  );
  const edgeColor =
    kind === 'pend'
      ? locked
        ? color.passed
        : color.caution
      : kind === 'sup'
        ? color.caution
        : color.accentDeep;
  const opacity = kind === 'pend' && !locked ? 0.4 + 0.5 * pulse : 0.85;
  return (
    <mesh position={mid} quaternion={quat}>
      <cylinderGeometry args={[0.016, 0.016, len, 6]} />
      <meshBasicMaterial color={edgeColor} transparent opacity={opacity} />
    </mesh>
  );
};

const Neuron: React.FC<{
  i: number;
  ignite: number;
  active: boolean;
  superseded: number;
}> = ({i, ignite, active, superseded}) => {
  const base = 0.09 + (i % 3) * 0.025;
  const scale = ignite * (active ? 2.1 : 1);
  const fade = 1 - superseded * 0.8;
  return (
    <mesh position={NODES[i]} scale={scale}>
      <icosahedronGeometry args={[base, 2]} />
      <meshStandardMaterial
        color={color.ink}
        emissive={active ? color.accentSoft : color.accent}
        emissiveIntensity={active ? 2.2 : 0.55}
        transparent
        opacity={fade}
      />
    </mesh>
  );
};

export const Brain3D: React.FC<{
  /** 0..1 — neurons pop in staggered */
  ignite: number;
  /** 0..1 — edges grow in order */
  edgeT: number;
  /** index of the neuron answering a query, -1 none */
  activeNode?: number;
  /** 0..1 — how superseded node 0 is */
  supersedeT?: number;
  /** pending edge is locked (approved) */
  pendingLocked?: boolean;
  /** slow orbit */
  orbit?: number;
  /** pulse phase for pending edge */
  pulse?: number;
}> = ({ignite, edgeT, activeNode = -1, supersedeT = 0, pendingLocked = false, orbit = 0, pulse = 0.5}) => {
  return (
    <group rotation={[0.25, orbit, 0]}>
      {/* faint cortex hull — sells "brain", keeps nodes visually inside */}
      <mesh scale={[1.25, 1.0, 0.9]}>
        <sphereGeometry args={[2.05, 24, 16]} />
        <meshBasicMaterial
          color={color.accentDeep}
          wireframe
          transparent
          opacity={0.05 * ignite}
        />
      </mesh>
      {NODES.map((_, i) => (
        <Neuron
          key={i}
          i={i}
          ignite={Math.max(0, Math.min(1, ignite * (NODES.length + 2) - i))}
          active={i === activeNode}
          superseded={i === 0 ? supersedeT : 0}
        />
      ))}
      {EDGES.map(([a, b, kind], i) => {
        const per = 1 / EDGES.length;
        return (
          <Edge
            key={i}
            a={a}
            b={b}
            kind={kind}
            locked={pendingLocked}
            pulse={pulse}
            grow={Math.max(0, Math.min(1, (edgeT - i * per) / per))}
          />
        );
      })}
    </group>
  );
};

/** Canvas wrapper tuned for the brand — drop into any scene. */
export const BrainStage3D: React.FC<{
  cameraZ?: number;
  children: React.ReactNode;
}> = ({cameraZ = 4.6, children}) => (
  <Canvas
    camera={{position: [0, 0, cameraZ], fov: 45}}
    style={{position: 'absolute', inset: 0}}
    gl={{antialias: true, alpha: true}}
  >
    <ambientLight intensity={0.35} />
    <pointLight position={[3, 3, 4]} intensity={2.4} color={color.accentSoft} />
    {children}
  </Canvas>
);
