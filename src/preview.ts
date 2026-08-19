import { generateWorld, getVectoredCurrentAt, ProceduralWorld, checkObstacleCollision } from './shared/mapGenerator';
import { parseGenome, generateRandomGenome, getComplementaryString } from './biology/dna';
import { CreatureRenderer } from './render/creatureRenderer';
import { CreatureAgent } from './shared/types';

const canvas = document.getElementById('preview-canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;

const seedInput = document.getElementById('seed-input') as HTMLInputElement;
const btnGenerate = document.getElementById('btn-generate') as HTMLButtonElement;
const btnRandom = document.getElementById('btn-random') as HTMLButtonElement;

let world: ProceduralWorld;
let offscreenCanvas: HTMLCanvasElement | null = null;
let renderer: CreatureRenderer;

// Interactive Player Sandbox Agent
let playerAgent: CreatureAgent | null = null;
const keys: { [key: string]: boolean } = {};

function spawnPlayerAgent() {
  const g = generateRandomGenome(256);
  const anti = getComplementaryString(g);
  const pheno = parseGenome(g, anti);

  playerAgent = {
    id: 9999,
    speciesId: g,
    genome: g,
    antisense: anti,
    phenotype: pheno,
    px: 19200 / 2,
    py: 10800 / 2,
    vx: 0,
    vy: 0,
    headingAngle: -Math.PI / 2,
    omegaRot: 0,
    energy: 100,
    age: 0,
    generation: 1,
    adrenaline: 1.0,
    hasEaten: false,
    neuronStates: [],
    neuronActivations: [],
    bendAngle: 0.0
  };

  // Force camera centered around player initially
  panX = window.innerWidth / 2 - playerAgent.px * zoom;
  panY = window.innerHeight / 2 - playerAgent.py * zoom;
}

// --------------------------------------------------------------------------
// Biome Color Index & Offline Offscreen Terrain Caching
// --------------------------------------------------------------------------
function createBiomeCache(world: ProceduralWorld) {
  offscreenCanvas = document.createElement('canvas');
  offscreenCanvas.width = 240;
  offscreenCanvas.height = 135;
  const oCtx = offscreenCanvas.getContext('2d')!;

  for (let c = 0; c < 240; c++) {
    for (let r = 0; r < 135; r++) {
      const idx = c * 135 + r;
      const biome = world.biomes[idx];
      if (biome) {
        oCtx.fillStyle = biome.color;
        oCtx.fillRect(c, r, 1, 1);
      }
    }
  }
}

// --------------------------------------------------------------------------
// Interactive Flow Particle Tracker (Simulating Fluid Dynamics)
// --------------------------------------------------------------------------
interface FlowParticle {
  x: number;
  y: number;
  life: number;
}
const particles: FlowParticle[] = [];
for (let i = 0; i < 400; i++) {
  particles.push({
    x: Math.random() * 19200,
    y: Math.random() * 10800,
    life: Math.random() * 200
  });
}

// --------------------------------------------------------------------------
// Interactive Viewport Camera Navigation State
// --------------------------------------------------------------------------
let zoom = 0.04; // default zoomed out view
let panX = 0;
let panY = 0;
let isDragging = false;
let startX = 0;
let startY = 0;

function init() {
  resize();
  renderer = new CreatureRenderer(canvas);
  loadWorld(seedInput.value || 'ALIFE_BASIN_77A');
  centerCamera();
  animate();
}

function loadWorld(seed: string) {
  world = generateWorld(seed);
  createBiomeCache(world);
  spawnPlayerAgent();
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

// --------------------------------------------------------------------------
// Browser Event Listeners
// --------------------------------------------------------------------------
window.addEventListener('resize', () => {
  resize();
});

// Camera Navigation: Dragging
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

  const worldX = (mouseX - panX) / zoom;
  const worldY = (mouseY - panY) / zoom;

  if (e.deltaY < 0) {
    zoom *= zoomFactor;
  } else {
    zoom /= zoomFactor;
  }
  zoom = Math.max(0.01, Math.min(1.5, zoom));

  panX = mouseX - worldX * zoom;
  panY = mouseY - worldY * zoom;
});

// Keyboard Listeners for WASD / Arrow Keys Steering
window.addEventListener('keydown', (e) => {
  keys[e.code] = true;
  if (['KeyW', 'KeyA', 'KeyS', 'KeyD', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.code)) {
    e.preventDefault();
  }
});

window.addEventListener('keyup', (e) => {
  keys[e.code] = false;
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
  // 1. Physics Step for Player Sandbox Agent
  if (playerAgent) {
    let thrustInput = 0.0;
    let bendInput = 0.0;

    if (keys['KeyW'] || keys['ArrowUp']) thrustInput += 1.0;
    if (keys['KeyS'] || keys['ArrowDown']) thrustInput -= 1.0;
    if (keys['KeyA'] || keys['ArrowLeft']) bendInput += 1.0;  // bend left
    if (keys['KeyD'] || keys['ArrowRight']) bendInput -= 1.0; // bend right

    const stiffness = playerAgent.phenotype.stiffness;
    const pulse = playerAgent.phenotype.pulseSpeed;
    const meanRadius = playerAgent.phenotype.spinalHarmonics.meanRadius;
    const baseLength = playerAgent.phenotype.spinalHarmonics.baseLength;

    // Calculate maximum thrust capacity
    let thrustMag = stiffness * (pulse * 1000 * pulse * 1000) * 6.0;
    const wavePhase = playerAgent.phenotype.wavePhase;
    const etaSwim = Math.max(0.1, Math.min(3.2, (baseLength / (meanRadius * 3.5)) * Math.max(0.01, Math.sin(wavePhase)) * stiffness));
    thrustMag *= etaSwim;

    const netThrustForce = thrustInput * thrustMag;
    let fx = netThrustForce * Math.cos(playerAgent.headingAngle);
    let fy = netThrustForce * Math.sin(playerAgent.headingAngle);

    // Body Flexion Bending Angle
    const maxFlexion = 1.2;
    const targetBending = bendInput * (maxFlexion / Math.max(0.2, stiffness));
    playerAgent.bendAngle = playerAgent.bendAngle || 0.0;
    playerAgent.bendAngle += (targetBending - playerAgent.bendAngle) * 0.15;
    playerAgent.bendAngle = Math.max(-maxFlexion, Math.min(maxFlexion, playerAgent.bendAngle));

    const mass = Math.pow(meanRadius, 1.5) * (baseLength / 25);
    const vForward = playerAgent.vx * Math.cos(playerAgent.headingAngle) + playerAgent.vy * Math.sin(playerAgent.headingAngle);

    // Bending + acceleration = curve
    const curvatureFactor = 0.015;
    const deltaHeading = vForward * playerAgent.bendAngle * curvatureFactor;
    playerAgent.headingAngle += deltaHeading;
    playerAgent.headingAngle = Math.atan2(Math.sin(playerAgent.headingAngle), Math.cos(playerAgent.headingAngle));

    playerAgent.omegaRot = playerAgent.bendAngle / 12.0;

    const receptorBallast = playerAgent.phenotype.organelles.length * 0.18;
    const dragForward = (meanRadius * 0.015 + receptorBallast) * (1.0 - stiffness * 0.3);

    const dragForceForward = -dragForward * vForward;
    const fxDrag = dragForceForward * Math.cos(playerAgent.headingAngle);
    const fyDrag = dragForceForward * Math.sin(playerAgent.headingAngle);

    const ax = (fx + fxDrag) / mass;
    const ay = (fy + fyDrag) / mass;

    playerAgent.vx += ax;
    playerAgent.vy += ay;

    playerAgent.vx *= 0.94;
    playerAgent.vy *= 0.94;

    // Zero lateral slippage
    const netSpeed = playerAgent.vx * Math.cos(playerAgent.headingAngle) + playerAgent.vy * Math.sin(playerAgent.headingAngle);
    playerAgent.vx = netSpeed * Math.cos(playerAgent.headingAngle);
    playerAgent.vy = netSpeed * Math.sin(playerAgent.headingAngle);

    const current = getVectoredCurrentAt(world, playerAgent.px, playerAgent.py);
    playerAgent.vx += current.vx;
    playerAgent.vy += current.vy;

    playerAgent.px += playerAgent.vx;
    playerAgent.py += playerAgent.vy;

    // Hard boundary wall collisions for player
    const r = meanRadius;
    const restitution = 0.5;
    if (playerAgent.px < r) {
      playerAgent.px = r;
      playerAgent.vx = -Math.abs(playerAgent.vx) * restitution;
    } else if (playerAgent.px > 19200 - r) {
      playerAgent.px = 19200 - r;
      playerAgent.vx = -Math.abs(playerAgent.vx) * restitution;
    }
    if (playerAgent.py < r) {
      playerAgent.py = r;
      playerAgent.vy = -Math.abs(playerAgent.vy) * restitution;
    } else if (playerAgent.py > 10800 - r) {
      playerAgent.py = 10800 - r;
      playerAgent.vy = -Math.abs(playerAgent.vy) * restitution;
    }

    // Obstacles collision
    const obs_col = checkObstacleCollision(world, playerAgent.px, playerAgent.py, meanRadius);
    if (obs_col.collided) {
      playerAgent.px += obs_col.normalX * obs_col.overlap;
      playerAgent.py += obs_col.normalY * obs_col.overlap;
      const dot = playerAgent.vx * obs_col.normalX + playerAgent.vy * obs_col.normalY;
      playerAgent.vx = (playerAgent.vx - 2.0 * dot * obs_col.normalX) * 0.45;
      playerAgent.vy = (playerAgent.vy - 2.0 * dot * obs_col.normalY) * 0.45;
    }

    // Camera smoothly locks onto the player
    if (!isDragging) {
      const targetPanX = window.innerWidth / 2 - playerAgent.px * zoom;
      const targetPanY = window.innerHeight / 2 - playerAgent.py * zoom;
      panX += (targetPanX - panX) * 0.1;
      panY += (targetPanY - panY) * 0.1;
    }
  }

  // Clear Canvas
  ctx.fillStyle = '#020617';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.translate(panX, panY);
  ctx.scale(zoom, zoom);

  // 1. Draw Biome Areas
  if (offscreenCanvas) {
    ctx.save();
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(offscreenCanvas, 0, 0, 19200, 10800);
    ctx.restore();
  }

  // 2. Update and Draw Flow Particles
  ctx.fillStyle = '#0ea5e9';
  for (const p of particles) {
    const current = getVectoredCurrentAt(world, p.x, p.y);
    p.x += current.vx * 25;
    p.y += current.vy * 25;

    p.x = (p.x + 19200) % 19200;
    p.y = (p.y + 10800) % 10800;

    p.life--;
    if (p.life <= 0) {
      p.x = Math.random() * 19200;
      p.y = Math.random() * 10800;
      p.life = 100 + Math.random() * 100;
    }

    ctx.fillRect(p.x, p.y, 16, 16);
  }

  // 3. Draw Thermal Vents (Skip if strength is 0)
  for (const vent of world.vents) {
    if (vent.strength === 0) continue;
    ctx.beginPath();
    ctx.arc(vent.x, vent.y, vent.radius, 0, Math.PI * 2);
    ctx.strokeStyle = 'rgba(14, 165, 233, 0.2)';
    ctx.lineWidth = 12;
    ctx.setLineDash([40, 40]);
    ctx.stroke();
    ctx.setLineDash([]);

    ctx.beginPath();
    ctx.arc(vent.x, vent.y, 80, 0, Math.PI * 2);
    ctx.fillStyle = vent.forceType === 'push' ? '#38bdf8' : vent.forceType === 'pull' ? '#ec4899' : '#a855f7';
    ctx.fill();

    ctx.font = '240px monospace';
    ctx.fillStyle = 'rgba(148, 163, 184, 0.6)';
    ctx.textAlign = 'center';
    ctx.fillText(`${vent.forceType.toUpperCase()}`, vent.x, vent.y - 150);
  }

  // 4. Draw Solid Obstacles
  for (const obs of world.obstacles) {
    ctx.beginPath();
    ctx.arc(obs.x, obs.y, obs.radius + 30, 0, Math.PI * 2);
    ctx.fillStyle = obs.type === 'rock' ? 'rgba(51, 65, 85, 0.15)' : 'rgba(244, 63, 94, 0.15)';
    ctx.fill();

    ctx.beginPath();
    ctx.moveTo(obs.vertices[0].x, obs.vertices[0].y);
    for (let j = 1; j < obs.vertices.length; j++) {
      ctx.lineTo(obs.vertices[j].x, obs.vertices[j].y);
    }
    ctx.closePath();
    ctx.fillStyle = obs.color;
    ctx.fill();

    ctx.strokeStyle = '#f1f5f9';
    ctx.lineWidth = 15;
    ctx.stroke();

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

  // 6. Draw Player Agent (Steerable Sandbox Creature)
  if (playerAgent && renderer) {
    renderer.render(playerAgent.phenotype, Date.now() * 0.05, playerAgent.px, playerAgent.py, playerAgent.headingAngle, playerAgent.omegaRot);
  }

  ctx.restore();
}

// Start
init();
