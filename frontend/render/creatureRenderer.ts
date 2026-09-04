import { CreaturePhenotype, HSLColor } from "../shared/types";

function hslToString(color: HSLColor, lightnessModifier = 0, opacity = 1): string {
  const l = Math.max(0, Math.min(100, color.l + lightnessModifier));
  return `hsla(${color.h}, ${color.s}%, ${l}%, ${opacity})`;
}

function shiftHue(color: HSLColor, shift: number): HSLColor {
  return {
    h: (color.h + shift + 360) % 360,
    s: color.s,
    l: color.l
  };
}

// Seedable PRNG for anatomical noise
function mulberry32(a: number) {
  return function() {
    let t = a += 0x6D2B79F5;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export class CreatureRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(canvas: HTMLCanvasElement) {
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Could not get 2D context");
    }
    this.ctx = context;
  }

  /**
   * Calculates the local radius R(s, theta) of the continuous protoplasmic cylinder.
   * Combines base Fourier harmonics, peristaltic wave squeezing, and morphogenetic Gaussian deforms.
   */
  private getSpinalRadiusAt(
    s: number,              // Position along spine s in [0, 1]
    localTheta: number,     // Angle around spine theta (0 is facing right, Math.PI is left)
    phenotype: CreaturePhenotype,
    time: number
  ): number {
    const harmonics = phenotype.spinalHarmonics;
    
    // 1. Enforce Natural Biological Teardrop Tapering Envelope
    const sTaper = 1.0 - 0.72 * Math.pow(s, 1.4);

    // 2. Calculate Base Fourier Harmonics along s
    let modulation = 0;
    for (let j = 0; j < 4; j++) {
      const frequencyFactor = (j + 1) * Math.PI;
      modulation += harmonics.amplitudes[j] * Math.cos(frequencyFactor * s + harmonics.phases[j]);
    }
    
    // Combine tapering envelope with harmonics modulation
    const baseR = harmonics.meanRadius * sTaper * (1.0 + modulation);

    // 3. Apply Peristaltic Squeezing Waves
    const wavePhaseOffset = s * phenotype.wavePhase;
    const waveVal = Math.sin(time * phenotype.pulseSpeed - wavePhaseOffset);
    const contractionScale = 1.0 + 0.09 * waveVal; // Max ±9% radial pulse
    let r = baseR * contractionScale;

    // ==========================================================================
    // 4. Continuous Gaussian Membrane Deformations (Unbounded Sprouting Buds)
    // ==========================================================================
    phenotype.organelles.forEach(patch => {
      const targetAngle = ((90.0 - patch.angle) * Math.PI) / 180;
      
      const style = patch.expressionStyle;
      const aff = patch.spectralAffinity;
      const scale = patch.scale;

      // Muscular limb outgrowths sway with higher amplitude
      const wiggleMultiplier = style >= 0.72 ? 1.8 : 1.0;
      const wigglePhase = patch.spinalPos * 4.0 + targetAngle * 0.03;
      const waveFreq = aff > 0.7 ? 0.005 : 0.0035;
      const sway = phenotype.wiggleAmplitude * wiggleMultiplier * Math.sin(time * waveFreq + wigglePhase);
      const animatedTargetAngle = targetAngle + sway;

      let diffAngle = localTheta - animatedTargetAngle;
      while (diffAngle > Math.PI) diffAngle -= Math.PI * 2;
      while (diffAngle < -Math.PI) diffAngle += Math.PI * 2;

      const diffS = s - patch.spinalPos;
      const wLongitudinal = (style >= 0.72 ? 0.12 : 0.07) * scale;

      if (style >= 0.72) {
        // A. FLESHY MUSCULAR LIMB (Swimming Foot / Creeping Tentacle)
        const length = 38 * scale * style; 
        const wAngular = 0.22 + patch.bandwidth * 0.4; // thick muscular root
        
        const distSquared = (diffAngle * diffAngle) / (wAngular * wAngular) + (diffS * diffS) / (wLongitudinal * wLongitudinal);
        r += length * Math.exp(-distSquared);
      } else if (patch.spinalPos < 0.25) {
        // B. CEPHALIC SPROUTING BUD (Brain Lobe / Head Crest)
        const length = 22 * scale * (1.0 - style);
        const wAngular = 0.35 + patch.bandwidth * 0.45; // extremely broad head flare
        const wLongitudinal = 0.15;

        const distSquared = (diffAngle * diffAngle) / (wAngular * wAngular) + (diffS * diffS) / (wLongitudinal * wLongitudinal);
        r += length * Math.exp(-distSquared);
      } else {
        // C. CILIATED PROTRUSION (Cilia / Feeler / Sensory Hillock)
        const height = 18 * scale * style;
        const wAngular = 0.18 + patch.bandwidth * 0.22; // wider angular root for smooth, organic hillocks
        const distSquared = (diffAngle * diffAngle) / (wAngular * wAngular) + (diffS * diffS) / (wLongitudinal * wLongitudinal);
        r += height * Math.exp(-distSquared);
      }
    });

    return Math.max(2, r); // Keep radius safe
  }

  /**
   * Main Render Routine. Plots the entire organism as a single, continuous, closed vector envelope!
   */
  public render(
    phenotype: CreaturePhenotype,
    time = 0,
    px = 256,
    py = 256,
    headingAngle = -Math.PI / 2,
    omegaRot = 0
  ): void {
    this.ctx.imageSmoothingEnabled = true;

    // Reset shadow
    this.ctx.shadowBlur = 0;
    this.ctx.shadowColor = "transparent";

    const pColor = phenotype.primaryColor;
    const outlineColor = "hsl(240, 24%, 10%)";

    const L = phenotype.spinalHarmonics.baseLength;
    const meanRadius = phenotype.spinalHarmonics.meanRadius;

    // ==========================================================================
    // 2. THE HOLY GRAIL: SAVE CANVAS CONTEXT & ROTATE / TRANSLATE GLOBALLY
    // ==========================================================================
    // Aligns the entire coordinate system to the creature's heading.
    // The creature's body is drawn 100% inside local-space (straight up/down)
    // with absolutely zero orientation-dependent distortion bugs!
    this.ctx.save();
    this.ctx.translate(px, py);
    
    const thetaShift = headingAngle + Math.PI / 2; // align spine vertically
    this.ctx.rotate(thetaShift);
    
    // Scale down entire drawing by 50% to proportion creatures beautifully in the fullscreen tank
    this.ctx.scale(0.5, 0.5);

    // ==========================================================================
    // 3. Trace Notochord Spinal Center Centers inside Local Space
    // ==========================================================================
    const steps = 60;
    const spinalPoints: { x: number; y: number }[] = [];
    
    const leftEpidermisPoints: { x: number; y: number }[] = [];
    const rightEpidermisPoints: { x: number; y: number }[] = [];
    
    const leftCoelomPoints: { x: number; y: number }[] = [];
    const rightCoelomPoints: { x: number; y: number }[] = [];

    const curveAmp = phenotype.spinalHarmonics.spinalCurve;
    const curveFreq = phenotype.spinalHarmonics.spinalCurveFreq;
    const paraAmp = phenotype.spinalHarmonics.parapodiaAmp;
    const paraFreq = phenotype.spinalHarmonics.parapodiaFreq;
    const flatHead = phenotype.spinalHarmonics.flatteningHead;

    for (let j = 0; j <= steps; j++) {
      const s = j / steps;

      // Local spine coordinate (with continuous lateral bending + dynamic turn flexing!)
      const localY = -L / 2 + s * L;
      const rotBending = omegaRot * 280.0; // Scaled up to be beautifully visible!
      const localX = curveAmp * Math.sin(Math.PI * s * curveFreq) - rotBending * Math.sin(Math.PI * s);

      spinalPoints.push({ x: localX, y: localY });

      // Sample base left and right flanc radii
      let rLeft = this.getSpinalRadiusAt(s, Math.PI, phenotype, time);
      let rRight = this.getSpinalRadiusAt(s, 0, phenotype, time);

      // A. Parapodia / Lateral Fin Outgrowths (Undulating side paddles)
      if (paraAmp > 0) {
        const finWave = paraAmp * Math.sin(Math.PI * paraFreq * s + time * 0.008);
        const finOutgrowth = Math.max(0, finWave) * meanRadius * 0.95;
        rLeft += finOutgrowth;
        rRight += finOutgrowth;
      }

      // B. Regional Head Widening (Hammerhead / Planaria flatheads)
      if (flatHead !== 0 && s < 0.22) {
        const headWidenMultiplier = 1.0 + flatHead * Math.cos((s / 0.22) * Math.PI / 2);
        rLeft *= headWidenMultiplier;
        rRight *= headWidenMultiplier;
      }

      // C. HIGH-FREQUENCY PROTOPLASMIC SHIVERING WAVE (Soft Skin Membrane)
      const rippleLeft = 1.8 * Math.sin(time * 0.012 + s * 22.0);
      const rippleRight = 1.8 * Math.cos(time * 0.012 + s * 22.0); // asymmetric ripple
      rLeft += rippleLeft;
      rRight += rippleRight;

      // D. PREDATOR JAW SPINE MODULATION (Serrated Spike-Denticles)
      if (phenotype.isPredator && s < 0.22) {
        // High frequency serrated sharp saw-teeth spikes
        const sawTeeth = 6.0 * Math.max(0, Math.sin(s * 85.0)); 
        rLeft += sawTeeth;
        rRight += sawTeeth;
      }

      // ========================================================================
      // 4. ABSOLUTE STABLE BOUNDARIES (IN LOCAL COORDINATES)
      // ========================================================================
      // Left and Right flanc offsets are applied relative to local X,
      // completely immunizing them from global rotation flips!
      leftEpidermisPoints.push({ x: localX - rLeft, y: localY });
      rightEpidermisPoints.push({ x: localX + rRight, y: localY });

      leftCoelomPoints.push({ x: localX - rLeft * 0.75, y: localY });
      rightCoelomPoints.push({ x: localX + rRight * 0.75, y: localY });
    }

    // Aura glow centered around the local head (spinalPoint index 0)
    const headPt = spinalPoints[0];
    const auraGrad = this.ctx.createRadialGradient(headPt.x, headPt.y, meanRadius * 0.3, headPt.x, headPt.y, meanRadius * 4.2);
    auraGrad.addColorStop(0, hslToString(pColor, 10, 0.12));
    auraGrad.addColorStop(1, "rgba(15, 17, 26, 0)");
    this.ctx.fillStyle = auraGrad;
    this.ctx.beginPath();
    this.ctx.arc(headPt.x, headPt.y, meanRadius * 4.2, 0, Math.PI * 2);
    this.ctx.fill();

    // ==========================================
    // 5. BUILD LOCAL CLOSED VECTOR ENVELOPE PATHS (Flawless, no ccw flips!)
    // ==========================================
    const buildClosedEnvelopePath = (leftPoints: { x: number; y: number }[], rightPoints: { x: number; y: number }[], scaleFactor = 1.0): Path2D => {
      const path = new Path2D();
      
      // Start at head left
      path.moveTo(leftPoints[0].x, leftPoints[0].y);

      // Run down left flanc
      for (let j = 1; j <= steps; j++) {
        path.lineTo(leftPoints[j].x, leftPoints[j].y);
      }

      // Smoothly wrap tail tip cap (Constant clockwise half-circle from Left (PI) to Right (0))
      const tailCenter = spinalPoints[steps];
      const rTail = this.getSpinalRadiusAt(1.0, 0, phenotype, time) * scaleFactor;
      path.arc(tailCenter.x, tailCenter.y, rTail, Math.PI, 0, true); // Always clock-wise

      // Run up right flanc (backwards from tail to head)
      for (let j = steps; j >= 0; j--) {
        path.lineTo(rightPoints[j].x, rightPoints[j].y);
      }

      // Smoothly wrap head tip cap (Constant clockwise half-circle from Right (0) to Left (PI))
      const headCenter = spinalPoints[0];
      const headWidenMultiplier = flatHead !== 0 ? (1.0 + flatHead) : 1.0;
      const rHead = this.getSpinalRadiusAt(0.0, 0, phenotype, time) * headWidenMultiplier * scaleFactor;
      path.arc(headCenter.x, headCenter.y, rHead, 0, Math.PI, true); // Always clock-wise

      path.closePath();
      return path;
    };

    const epidermisEnvelope = buildClosedEnvelopePath(leftEpidermisPoints, rightEpidermisPoints, 1.0);
    const coelomEnvelope = buildClosedEnvelopePath(leftCoelomPoints, rightCoelomPoints, 0.75);

    // ==========================================
    // 6. DRAW MULTILAYERED PROTOPLASM
    // ==========================================
    // Pass 1: Outer epidermis boundary stroke
    this.ctx.strokeStyle = outlineColor;
    this.ctx.lineWidth = 14;
    this.ctx.lineJoin = "round";
    this.ctx.lineCap = "round";
    this.ctx.stroke(epidermisEnvelope);

    // Pass 2: Filled outer epidermis skin
    const tailPt = spinalPoints[steps];
    const epidermisGrad = this.ctx.createLinearGradient(headPt.x, headPt.y, tailPt.x, tailPt.y);
    epidermisGrad.addColorStop(0, hslToString(pColor, -12, 0.95));
    epidermisGrad.addColorStop(1, hslToString(pColor, -35, 0.95));
    this.ctx.fillStyle = epidermisGrad;
    this.ctx.fill(epidermisEnvelope);

    // Pass 3: Filled inner coelom core flesh (Jelly effect)
    const coelomGrad = this.ctx.createLinearGradient(headPt.x, headPt.y, tailPt.x, tailPt.y);
    coelomGrad.addColorStop(0, hslToString(pColor, 18, 0.72));
    coelomGrad.addColorStop(0.5, hslToString(pColor, 10, 0.72));
    coelomGrad.addColorStop(1, hslToString(pColor, -8, 0.72));
    this.ctx.fillStyle = coelomGrad;
    this.ctx.fill(coelomEnvelope);

    this.ctx.save();
    this.ctx.clip(coelomEnvelope);

    // ==========================================
    // 7. DRAW VISCERAL INTERNAL ORGANS (IN LOCAL SPACE)
    // ==========================================
    
    // A. Digestive Gut Tube (Magen-Darm-Kanal)
    this.ctx.beginPath();
    this.ctx.moveTo(spinalPoints[0].x, spinalPoints[0].y);
    for (let j = 1; j <= steps; j++) {
      this.ctx.lineTo(spinalPoints[j].x, spinalPoints[j].y);
    }
    
    const gutGrad = this.ctx.createLinearGradient(headPt.x, headPt.y, tailPt.x, tailPt.y);
    gutGrad.addColorStop(0, "rgba(245, 158, 11, 0.7)");
    gutGrad.addColorStop(0.4, "rgba(245, 158, 11, 0.5)");
    gutGrad.addColorStop(1, "rgba(21, 128, 61, 0.35)");
    
    this.ctx.strokeStyle = gutGrad;
    this.ctx.lineWidth = meanRadius * 0.28;
    this.ctx.lineCap = "round";
    this.ctx.lineJoin = "round";
    this.ctx.stroke();

    // B. Pulsating Dorsal Blood Vessel (Kreislauf)
    this.ctx.beginPath();
    this.ctx.moveTo(spinalPoints[0].x, spinalPoints[0].y);
    for (let j = 1; j <= steps; j++) {
      const s = j / steps;
      const heartbeat = 1.0 + 0.3 * Math.sin(time * 0.008 - s * 4.0);
      this.ctx.lineWidth = meanRadius * 0.06 * heartbeat;
      this.ctx.lineTo(spinalPoints[j].x, spinalPoints[j].y);
    }
    this.ctx.strokeStyle = "rgba(239, 68, 68, 0.85)";
    this.ctx.stroke();

    // C. Segmented Cartilage Vertebrae (Spine Skeleton)
    const vertCount = 18;
    for (let i = 0; i <= vertCount; i++) {
      const idx = Math.round((i / vertCount) * steps);
      const sPt = spinalPoints[idx];
      const s = idx / steps;
      
      const sTaper = 1.0 - 0.72 * Math.pow(s, 1.4);
      const vRadius = meanRadius * sTaper * 0.32;

      this.ctx.save();
      this.ctx.translate(sPt.x, sPt.y);
      // No headingAngle rotation needed! Canvas context handles it globally!

      this.ctx.beginPath();
      this.ctx.moveTo(0, -vRadius * 0.6);
      this.ctx.lineTo(vRadius * 0.8, 0);
      this.ctx.lineTo(0, vRadius * 0.6);
      this.ctx.lineTo(-vRadius * 0.8, 0);
      this.ctx.closePath();

      this.ctx.fillStyle = "rgba(254, 250, 240, 0.82)";
      this.ctx.strokeStyle = "rgba(240, 230, 210, 0.9)";
      this.ctx.lineWidth = 1.2;
      this.ctx.fill();
      this.ctx.stroke();

      this.ctx.restore();
    }

    // D. Bioluminescent Oocytes / Floating Gonads
    const rand = mulberry32(phenotype.bodySeed);
    const eggCount = 8 + Math.floor(rand() * 6);
    this.ctx.fillStyle = rand() > 0.5 ? "rgba(217, 70, 239, 0.85)" : "rgba(34, 211, 238, 0.85)";
    for (let k = 0; k < eggCount; k++) {
      const eggS = 0.2 + rand() * 0.6;
      const sIdx = Math.round(eggS * steps);
      const basePt = spinalPoints[sIdx];
      
      const rAtS = this.getSpinalRadiusAt(eggS, 0, phenotype, time);
      const offsetLimit = rAtS * 0.38;
      
      const lateralOffset = (rand() * 2.0 - 1.0) * offsetLimit;
      const floatOffset = 3.5 * Math.sin(time * 0.0022 + k);
      
      // Projects lateral eggs utilizing orientation-stable local normal axes (local X direction)
      const eggX = basePt.x + lateralOffset; // Left flanc is always negative local X
      const eggY = basePt.y + floatOffset;

      this.ctx.beginPath();
      this.ctx.arc(eggX, eggY, 2.5, 0, Math.PI * 2);
      this.ctx.fill();
      
      this.ctx.strokeStyle = "#ffffff";
      this.ctx.lineWidth = 0.5;
      this.ctx.stroke();
    }

    this.ctx.restore(); // end coelom clip

    // ==========================================================================
    // 8. DRAW INTERNAL SKELETAL RIDGES FOR FLESHY LIMBS (`expressionStyle >= 0.72`)
    // ==========================================================================
    phenotype.organelles.forEach(patch => {
      if (patch.expressionStyle >= 0.72) {
        const targetAngle = ((90.0 - patch.angle) * Math.PI) / 180;
        
        const wigglePhase = patch.spinalPos * 4.0 + targetAngle * 0.03;
        const waveFreq = patch.spectralAffinity > 0.7 ? 0.005 : 0.0035;
        const sway = phenotype.wiggleAmplitude * 1.8 * Math.sin(time * waveFreq + wigglePhase);
        const animatedAngle = targetAngle + sway;

        const sIdx = Math.round(patch.spinalPos * steps);
        const sCenter = spinalPoints[sIdx];
        
        let sRadius = this.getSpinalRadiusAt(patch.spinalPos, animatedAngle, phenotype, time);
        if (paraAmp > 0) {
          const finWave = paraAmp * Math.sin(Math.PI * paraFreq * patch.spinalPos + time * 0.008);
          sRadius += Math.max(0, finWave) * meanRadius * 0.95;
        }
        if (flatHead !== 0 && patch.spinalPos < 0.22) {
          sRadius *= (1.0 + flatHead * Math.cos((patch.spinalPos / 0.22) * Math.PI / 2));
        }

        const ptTip = {
          x: sCenter.x + sRadius * Math.cos(animatedAngle),
          y: sCenter.y + sRadius * Math.sin(animatedAngle)
        };

        this.ctx.beginPath();
        this.ctx.moveTo(sCenter.x, sCenter.y);
        const midX = (sCenter.x + ptTip.x) / 2 + 6 * Math.cos(animatedAngle + Math.PI / 2);
        const midY = (sCenter.y + ptTip.y) / 2 + 6 * Math.sin(animatedAngle + Math.PI / 2);
        this.ctx.quadraticCurveTo(midX, midY, ptTip.x, ptTip.y);

        this.ctx.strokeStyle = "rgba(255, 255, 255, 0.28)";
        this.ctx.lineWidth = 4 * patch.scale;
        this.ctx.lineCap = "round";
        this.ctx.stroke();

        this.ctx.beginPath();
        this.ctx.moveTo(sCenter.x, sCenter.y);
        this.ctx.lineTo(ptTip.x + 5 * Math.cos(animatedAngle - Math.PI / 4), ptTip.y + 5 * Math.sin(animatedAngle - Math.PI / 4));
        this.ctx.moveTo(sCenter.x, sCenter.y);
        this.ctx.lineTo(ptTip.x + 5 * Math.cos(animatedAngle + Math.PI / 4), ptTip.y + 5 * Math.sin(animatedAngle + Math.PI / 4));
        this.ctx.strokeStyle = "rgba(255, 255, 255, 0.1)";
        this.ctx.lineWidth = 1.5;
        this.ctx.stroke();
      }
    });

    // ==========================================
    // 9. DRAW GLOSSY SPECTRAL HIGHLIGHTS ON ENVELOPE
    // ==========================================
    phenotype.organelles.forEach(patch => {
      const targetAngle = ((90.0 - patch.angle) * Math.PI) / 180;
      
      const wigglePhase = patch.spinalPos * 4.0 + targetAngle * 0.03;
      const waveFreq = patch.spectralAffinity > 0.7 ? 0.005 : 0.0035;
      const sway = phenotype.wiggleAmplitude * Math.sin(time * waveFreq + wigglePhase);
      const animatedAngle = targetAngle + sway;

      const sIdx = Math.round(patch.spinalPos * steps);
      const sCenter = spinalPoints[sIdx];
      
      let sRadius = this.getSpinalRadiusAt(patch.spinalPos, animatedAngle, phenotype, time);
      
      if (paraAmp > 0) {
        const finWave = paraAmp * Math.sin(Math.PI * paraFreq * patch.spinalPos + time * 0.008);
        sRadius += Math.max(0, finWave) * meanRadius * 0.95;
      }
      if (flatHead !== 0 && patch.spinalPos < 0.22) {
        const headWidenMultiplier = 1.0 + flatHead * Math.cos((patch.spinalPos / 0.22) * Math.PI / 2);
        sRadius *= headWidenMultiplier;
      }

      // Protrude relative to local coordinates (no global rotation shifts)
      const pt = {
        x: sCenter.x + sRadius * Math.cos(animatedAngle),
        y: sCenter.y + sRadius * Math.sin(animatedAngle)
      };

      const normal = animatedAngle;

      this.ctx.save();
      this.ctx.translate(pt.x, pt.y);
      this.ctx.rotate(normal - Math.PI / 2);

      const scale = patch.scale;
      const aff = patch.spectralAffinity;
      const style = patch.expressionStyle;
      const organelleColor = shiftHue(pColor, patch.hueShift);

      if (style < 0.3) {
        const baseWidth = 10 * scale;
        this.ctx.beginPath();
        this.ctx.arc(0, 0, baseWidth - 1, Math.PI, 0);
        this.ctx.closePath();
        
        if (aff >= 0.8) {
          const lensGrad = this.ctx.createRadialGradient(-baseWidth * 0.2, -baseWidth * 0.3, 1, 0, 0, baseWidth);
          lensGrad.addColorStop(0, "#ffffff");
          lensGrad.addColorStop(0.3, `hsl(${organelleColor.h}, 100%, 75%)`);
          lensGrad.addColorStop(1, "rgba(255,255,255,0)");
          this.ctx.fillStyle = lensGrad;
          this.ctx.fill();
          
          this.ctx.beginPath();
          this.ctx.arc(-baseWidth * 0.4, -baseWidth * 0.4, 2.2 * scale, 0, Math.PI * 2);
          this.ctx.fillStyle = "#ffffff";
          this.ctx.fill();
        } else if (aff > 0.65 && aff < 0.8) {
          const pitGrad = this.ctx.createRadialGradient(0, 0, 1, 0, 0, baseWidth);
          pitGrad.addColorStop(0, "#000000");
          pitGrad.addColorStop(0.5, "hsl(350, 95%, 45%)");
          pitGrad.addColorStop(1, "rgba(255,0,0,0)");
          this.ctx.fillStyle = pitGrad;
          this.ctx.fill();
        }
      } else if (style < 0.72) {
        if (patch.bandwidth < 0.35) {
          const endX = 0;
          const endY = -3.5 * scale; // Draw directly on the surface, tangent to the skin hillock

          this.ctx.beginPath();
          this.ctx.arc(endX, endY, 3.5 * scale, 0, Math.PI * 2);
          
          let beadColor = "";
          if (aff >= 0.8) beadColor = "#ffffff";
          else if (aff < 0.25) beadColor = "hsl(40, 100%, 70%)";
          else if (aff >= 0.25 && aff <= 0.65) beadColor = "hsl(120, 100%, 70%)";
          else beadColor = "hsl(340, 100%, 70%)";

          this.ctx.fillStyle = beadColor;
          this.ctx.shadowColor = beadColor;
          this.ctx.shadowBlur = 10;
          this.ctx.fill();
        }
      } else {
        const tipRadius = 5 * scale;
        this.ctx.beginPath();
        this.ctx.arc(0, -6, tipRadius, 0, Math.PI * 2);
        
        const tipColor = `hsl(${organelleColor.h}, 95%, 65%)`;
        this.ctx.fillStyle = tipColor;
        this.ctx.shadowColor = tipColor;
        this.ctx.shadowBlur = 8;
        this.ctx.fill();
      }
      this.ctx.restore();
    });

    // Trailing Prey Sensilla Ribbons (Elegance flow for Beute)
    if (!phenotype.isPredator) {
      this.ctx.save();
      this.ctx.strokeStyle = "rgba(0, 242, 254, 0.38)";
      this.ctx.lineWidth = 1.6;
      this.ctx.shadowColor = "#00f2fe";
      this.ctx.shadowBlur = 8;
      
      const tailLeft = leftEpidermisPoints[steps];
      const tailRight = rightEpidermisPoints[steps];

      // Draw two elegant wavy trails extending behind the tail tip
      const waveOffset1 = 12 * Math.sin(time * 0.005);
      const waveOffset2 = 12 * Math.cos(time * 0.005);

      this.ctx.beginPath();
      this.ctx.moveTo(tailLeft.x, tailLeft.y);
      this.ctx.bezierCurveTo(
        tailLeft.x - 12, tailLeft.y + 20,
        tailLeft.x - 24 + waveOffset1, tailLeft.y + 45,
        tailLeft.x - 16 + waveOffset1, tailLeft.y + 70
      );
      this.ctx.stroke();

      this.ctx.beginPath();
      this.ctx.moveTo(tailRight.x, tailRight.y);
      this.ctx.bezierCurveTo(
        tailRight.x + 12, tailRight.y + 20,
        tailRight.x + 24 + waveOffset2, tailRight.y + 45,
        tailRight.x + 16 + waveOffset2, tailRight.y + 70
      );
      this.ctx.stroke();
      this.ctx.restore();
    }

    // ==========================================
    // 10. RESTORE GLOBAL CANVAS CONTEXT
    // ==========================================
    this.ctx.restore();
  }
}
