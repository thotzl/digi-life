// Warp slider mapping shared by the Ocean and trainer speed sliders (TCK-160): slider positions 1..10 are 1x..10x and
// position 11 is the engine maximum of 35 steps per frame (invariant 2, `MAX_STEPS_PER_FRAME`). The engine clamps and
// echoes the warp (TCK-156), so every echoed value above 10 maps back to the maximum position.

export const WARP_MAX = 35;
export const WARP_SLIDER_MAX = 11;

export function sliderToWarp(position: number): number {
  return position <= 10 ? position : WARP_MAX;
}

export function warpToSlider(warp: number): number {
  return warp <= 10 ? warp : WARP_SLIDER_MAX;
}

export function warpLabel(position: number): string {
  return position <= 10 ? `${position}x` : `Max (${WARP_MAX}x)`;
}
