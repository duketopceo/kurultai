const EPS = 1e-8;
const MAX_DEPTH = 12;

interface Body {
  x: number;
  y: number;
  z: number;
}

class Cell {
  cx: number;
  cy: number;
  cz: number;
  half: number;
  mass = 0;
  mx = 0;
  my = 0;
  mz = 0;
  indices: number[] = [];
  children: Cell[] | null = null;

  constructor(cx: number, cy: number, cz: number, half: number) {
    this.cx = cx;
    this.cy = cy;
    this.cz = cz;
    this.half = half;
  }

  insert(bodies: Body[], i: number, depth: number): void {
    const b = bodies[i];
    this.mass += 1;
    this.mx += b.x;
    this.my += b.y;
    this.mz += b.z;

    if (this.children) {
      this.childFor(b).insert(bodies, i, depth + 1);
      return;
    }

    this.indices.push(i);
    if (this.indices.length <= 1 || depth >= MAX_DEPTH || this.half < EPS) return;

    this.subdivide();
    const pending = this.indices;
    this.indices = [];
    for (const idx of pending) this.childFor(bodies[idx]).insert(bodies, idx, depth + 1);
  }

  private subdivide(): void {
    const h = this.half * 0.5;
    const kids: Cell[] = new Array(8);
    for (let oct = 0; oct < 8; oct++) {
      const ox = oct & 1 ? h : -h;
      const oy = oct & 2 ? h : -h;
      const oz = oct & 4 ? h : -h;
      kids[oct] = new Cell(this.cx + ox, this.cy + oy, this.cz + oz, h);
    }
    this.children = kids;
  }

  private childFor(b: Body): Cell {
    const kids = this.children!;
    const oct =
      (b.x >= this.cx ? 1 : 0) | (b.y >= this.cy ? 2 : 0) | (b.z >= this.cz ? 4 : 0);
    return kids[oct];
  }

  accumulate(bodies: Body[], i: number, theta: number, out: { x: number; y: number; z: number }): void {
    if (this.mass === 0) return;
    const p = bodies[i];

    if (!this.children) {
      for (const j of this.indices) {
        if (j === i) continue;
        addRepel(p, bodies[j].x, bodies[j].y, bodies[j].z, 1, out);
      }
      return;
    }

    const comX = this.mx / this.mass;
    const comY = this.my / this.mass;
    const comZ = this.mz / this.mass;
    const dx = p.x - comX;
    const dy = p.y - comY;
    const dz = p.z - comZ;
    const d = Math.sqrt(dx * dx + dy * dy + dz * dz) + EPS;
    if ((this.half * 2) / d < theta) {
      addRepel(p, comX, comY, comZ, this.mass, out);
      return;
    }
    for (const child of this.children) child.accumulate(bodies, i, theta, out);
  }
}

function addRepel(
  p: Body,
  qx: number,
  qy: number,
  qz: number,
  mass: number,
  out: { x: number; y: number; z: number },
): void {
  const dx = p.x - qx;
  const dy = p.y - qy;
  const dz = p.z - qz;
  const d2 = dx * dx + dy * dy + dz * dz + EPS;
  const inv = mass / (d2 * Math.sqrt(d2));
  out.x += dx * inv;
  out.y += dy * inv;
  out.z += dz * inv;
}

export class Octree {
  private bodies: Body[];
  private theta: number;
  private root: Cell | null;

  constructor(bodies: Body[], theta: number) {
    this.bodies = bodies;
    this.theta = theta;
    if (bodies.length === 0) {
      this.root = null;
      return;
    }
    let minX = Infinity;
    let minY = Infinity;
    let minZ = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    let maxZ = -Infinity;
    for (const b of bodies) {
      if (b.x < minX) minX = b.x;
      if (b.y < minY) minY = b.y;
      if (b.z < minZ) minZ = b.z;
      if (b.x > maxX) maxX = b.x;
      if (b.y > maxY) maxY = b.y;
      if (b.z > maxZ) maxZ = b.z;
    }
    const cx = (minX + maxX) * 0.5;
    const cy = (minY + maxY) * 0.5;
    const cz = (minZ + maxZ) * 0.5;
    const half = Math.max(maxX - minX, maxY - minY, maxZ - minZ) * 0.5 + 1e-4;
    this.root = new Cell(cx, cy, cz, half);
    for (let i = 0; i < bodies.length; i++) this.root.insert(bodies, i, 0);
  }

  accumulate(i: number, out: { x: number; y: number; z: number }): void {
    out.x = 0;
    out.y = 0;
    out.z = 0;
    this.root?.accumulate(this.bodies, i, this.theta, out);
  }
}
