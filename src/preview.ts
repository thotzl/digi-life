import { generateWorld, getVectoredCurrentAt, ProceduralWorld } from './shared/mapGenerator';

const canvas = document.getElementById('preview-canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;

const seedInput = document.getElementById('seed-input') as HTMLInputElement;
const btnGenerate = document.getElementById('btn-generate') as HTMLButtonElement;
const btnRandom = document.getElementById('btn-random') as HTMLButtonElement;

let world: ProceduralWorld;
let zoom = 0.05; // Zoom out to see the large 19200x10800 map
let panX = 0;
let panY = 0;
let isDragging = false;
let startX = 0;
let startY = 0;

// Flow particles to visualize vectors
interface FlowParticle {
  x: number;
  y: number;
  life: number;
}
const particles: FlowParticle[] = [];
for (let i = 0; i < 500; i++) {
  particles.push({
    x: Math.random() * 19200,
    y: Math.random() * 10800,
    life: Math.random() * 200
  });
}

function init() {
  resize();
  loadWorld(seedInput.value);
  centerCamera();
  animate();
}

function loadWorld(seed: string) {
  world = generateWorld(seed);
}

function centerCamera() {
  const scale = Math.min(canvas.width / 19200, canvas.height / 10800);
  zoom = scale * 0.95;
  panX = (canvas.width - 19200 * zoom) / 2;
  panY = (canvas.height - 10800 * zoom) / 2;
}

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}

window.addEventListener('resize', () => {
  resize();
  centerCamera();
});

// Camera Navigation: Panning
canvas.addEventListener('mousedown', (e) => {
  isDragging = true;
  startX = e.clientX - panX;
  startY = e.clientY - panY;
});

window.addEventListener('mousemove', (e) => {
  if (isDragging) {
    panX = e.clientX - startX;
    panY = e.clientY - startY;
  }
});

window.addEventListener('mouseup', () => {
  isDragging = false;
});

// Camera Navigation: Zooming
canvas.addEventListener('wheel', (e) => {
  e.preventDefault();
  const zoomFactor = 1.1;
  const mouseX = e.clientX;
  const mouseY = e.clientY;

  // Convert mouse position to world coordinates before zoom
  const worldX = (mouseX - panX) / zoom;
  const worldY = (mouseY - panY) / zoom;

  if (e.deltaY < 0) {
    zoom *= zoomFactor;
  } else {
    zoom /= zoomFactor;
  }
  zoom = Math.max(0.01, Math.min(1.5, zoom));

  // Recalculate pan so zooming centers on mouse pointer
  panX = mouseX - worldX * zoom;
  panY = mouseY - worldY * zoom;
});

// Interaction Button Handlers
btnGenerate.addEventListener('click', () => {
  loadWorld(seedInput.value);
});

btnRandom.addEventListener('click', () => {
  const alph = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let randomSeed = '';
  for (let i = 0; i < 8; i++) {
    randomSeed += alph[Math.floor(Math.random() * alph.length)];
  }
  seedInput.value = randomSeed;
  loadWorld(randomSeed);
});

// --------------------------------------------------------------------------
// Animation Loop (60 FPS fluid rendering)
// --------------------------------------------------------------------------
function animate() {
  requestAnimationFrame(animate);
  draw();
}

function draw() {
  // Clear
  ctx.fillStyle = '#020617';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.translate(panX, panY);
  ctx.scale(zoom, zoom);

  // 1. Draw Biome Areas
  for (const biome of world.biomes) {
    ctx.fillStyle = biome.color;
    ctx.fillRect(biome.x, biome.y, biome.width, biome.height);

    // Biome boundary line
    ctx.strokeStyle = 'rgba(51, 65, 85, 0.4)';
    ctx.lineWidth = 40;
    ctx.strokeRect(biome.x, biome.y, biome.width, biome.height);

    // Draw Biome Labels (spaced vertically)
    ctx.font = 'bold 360px monospace';
    ctx.fillStyle = 'rgba(148, 163, 184, 0.15)';
    ctx.textAlign = 'center';
    ctx.fillText(biome.name, biome.x + biome.width / 2, biome.y + 1200);
  }

  // 2. Update and Draw Flow Particles (fluid currents)
  ctx.fillStyle = '#0ea5e9';
  for (const p of particles) {
    const current = getVectoredCurrentAt(world, p.x, p.y);
    p.x += current.vx * 25; // amplify speed for visualization
    p.y += current.vy * 25;

    // Toroidal warp particles
    p.x = (p.x + 19200) % 19200;
    p.y = (p.y + 10800) % 10800;

    p.life--;
    if (p.life <= 0) {
      p.x = Math.random() * 19200;
      p.y = Math.random() * 10800;
      p.life = 100 + Math.random() * 100;
    }

    // Render flow particle dot
    ctx.fillRect(p.x, p.y, 16, 16);
  }

  // 3. Draw Thermal Vents (Circular dotted centers)
  for (const vent of world.vents) {
    ctx.beginPath();
    ctx.arc(vent.x, vent.y, vent.radius, 0, Math.PI * 2);
    ctx.strokeStyle = 'rgba(14, 165, 233, 0.2)';
    ctx.lineWidth = 12;
    ctx.setLineDash([40, 40]);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw center core
    ctx.beginPath();
    ctx.arc(vent.x, vent.y, 80, 0, Math.PI * 2);
    ctx.fillStyle = vent.forceType === 'push' ? '#38bdf8' : vent.forceType === 'pull' ? '#ec4899' : '#a855f7';
    ctx.fill();

    // Vent Label
    ctx.font = '240px monospace';
    ctx.fillStyle = 'rgba(148, 163, 184, 0.6)';
    ctx.textAlign = 'center';
    ctx.fillText(`${vent.forceType.toUpperCase()}`, vent.x, vent.y - 150);
  }

  // 4. Draw Solid Obstacles (Reef / Rock barriers)
  for (const obs of world.obstacles) {
    // Soft outer glow ring
    ctx.beginPath();
    ctx.arc(obs.x, obs.y, obs.radius + 30, 0, Math.PI * 2);
    ctx.fillStyle = obs.type === 'rock' ? 'rgba(51, 65, 85, 0.15)' : 'rgba(244, 63, 94, 0.15)';
    ctx.fill();

    // Solid core
    ctx.beginPath();
    ctx.arc(obs.x, obs.y, obs.radius, 0, Math.PI * 2);
    ctx.fillStyle = obs.color;
    ctx.fill();

    // High tech stroke ring
    ctx.strokeStyle = '#f1f5f9';
    ctx.lineWidth = 15;
    ctx.stroke();

    // Barrier index label
    ctx.font = 'bold 120px monospace';
    ctx.fillStyle = '#1e293b';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`#${obs.id}`, obs.x, obs.y);
  }

  // 5. Draw world Outer Bounds boundary
  ctx.strokeStyle = 'rgba(14, 165, 233, 0.8)';
  ctx.lineWidth = 80;
  ctx.strokeRect(0, 0, 19200, 10800);

  ctx.restore();
}

// Start
init();
