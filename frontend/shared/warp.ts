// Warp slider mapping shared by the Ocean and trainer speed sliders (TCK-160; formerly copied into both): slider
// positions 1..10 are 1x..10x, 11 is 50x, 12 is "unlimited" (9999; the engine caps steps per frame, invariant 2).

export const WARP_SLIDER_MAX = 12;

export function sliderToWarp(position: number): number {
  if (position <= 10) return position;
  return position === 11 ? 50 : 9999;
}

export function warpToSlider(warp: number): number {
  if (warp <= 10) return warp;
  return warp === 50 ? 11 : 12;
}

export function warpLabel(position: number): string {
  if (position <= 10) return `${position}x`;
  return position === 11 ? "50x" : "🏎️ Unlimited";
}
