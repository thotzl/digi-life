// Display labels of a creature shared by roster, catalogue, pickers and trainer tiles (TCK-160; formerly copied into
// each view with different thresholds and a colour guessed from `genome.charCodeAt(17)`).
import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import type { HSLColor } from "../../bindings/HSLColor";

import { CARNIVORE_MIN, HERBIVORE_MAX } from "../../bindings/diet";

/** Diet thresholds of the backend `DietClass::from_carnivory` (`src/biology/diet.rs`, generated `bindings/diet.ts`). */
export { CARNIVORE_MIN, HERBIVORE_MAX };

export type DietClass = "herbivore" | "omnivore" | "carnivore";

export function dietClass(carnivory: number): DietClass {
  if (carnivory >= CARNIVORE_MIN) return "carnivore";
  if (carnivory >= HERBIVORE_MAX) return "omnivore";
  return "herbivore";
}

export const DIET_LABELS: Record<DietClass, string> = {
  herbivore: "Herbivore",
  omnivore: "Omnivore",
  carnivore: "Predator",
};

export function dietLabel(carnivory: number): string {
  return DIET_LABELS[dietClass(carnivory)];
}

/** Badge colour of a creature: the hue of its phenotype's primary colour at a fixed glow saturation/lightness;
 * neutral grey while the phenotype is unknown. */
export function badgeColor(phenotype: Pick<CreaturePhenotype, "primaryColor"> | null | undefined, alpha = 1): string {
  if (!phenotype) return `hsla(215, 16%, 47%, ${alpha})`;
  return hueColor(phenotype.primaryColor, alpha);
}

export function hueColor(color: HSLColor, alpha = 1): string {
  return `hsla(${color.h}, 85%, 55%, ${alpha})`;
}
