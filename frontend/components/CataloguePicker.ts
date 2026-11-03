// Catalogue picker (TCK-160): preview + card list of saved creatures for dialogs that take one of them (Ocean
// "Load from Catalog", trainer "Assign From Catalogue"). Formerly copied into both with its own preview loop.
import type { CatalogueCreature } from "../../bindings/CatalogueCreature";
import { getPhenotype, phenotypeCache, phenotypeVersion } from "../core/phenotypes";
import type { Scope } from "../core/scope";
import { styled } from "../core/styled";
import { badgeColor, dietLabel } from "../shared/creatureLabels";
import { CreaturePreview } from "./CreaturePreview";

const PreviewRow = styled("div")`
  display: flex;
  justify-content: center;
  align-items: center;
  background: rgba(2, 6, 23, 0.5);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 6px;
  padding: 12px;
  gap: 16px;
`;

const Meta = styled("div")`
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;

  b { font-size: 0.8rem; color: #fff; }
  span { font-size: 0.58rem; color: #64748b; line-height: 1.4; }
  em { font-style: normal; color: var(--modal-accent); }
`;

const Deck = styled("div")`
  height: 140px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 6px;
  background: rgba(15, 23, 42, 0.3);
  padding: 6px;
  box-sizing: border-box;

  &::-webkit-scrollbar { width: 3px; }
  &::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.15); border-radius: 2px; }
`;

const Card = styled("div")`
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 4px;
  padding: 8px 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.65rem;
  background: rgba(30, 41, 59, 0.15);
  transition: all 0.15s ease-in-out;

  &:hover { border-color: var(--modal-accent); }
  &.active { border-color: var(--modal-accent); background: rgba(148, 163, 184, 0.06); }
`;

const CardName = styled("div")`
  display: flex;
  align-items: center;
  gap: 8px;

  b { color: #fff; }
`;

const Dot = styled("div")`
  width: 6px;
  height: 6px;
  border-radius: 50%;
`;

const CardMeta = styled("span")`
  color: #64748b;
  font-size: 0.58rem;
  text-transform: uppercase;
`;

export class CataloguePicker {
  readonly element: HTMLElement;
  private index = 0;
  private readonly preview: CreaturePreview;
  private readonly name: HTMLElement;
  private readonly details: HTMLElement;
  private readonly deck: HTMLElement;

  /** `creatures` must not be empty; the preview animates inside `scope` (the modal's scope). */
  constructor(private readonly creatures: CatalogueCreature[], scope: Scope) {
    this.preview = new CreaturePreview({
      size: 72,
      bodyFraction: 0.72,
      style: { background: "#020617", border: "1.5px solid var(--modal-accent)", borderRadius: "50%", flexShrink: "0" },
    });
    this.name = document.createElement("b");
    this.details = document.createElement("span");
    this.deck = Deck();
    this.element = document.createElement("div");
    this.element.style.cssText = "display:flex; flex-direction:column; gap:14px;";
    this.element.appendChild(PreviewRow({ children: [this.preview.element, Meta({ children: [this.name, this.details] })] }));
    this.element.appendChild(this.deck);

    this.preview.start(scope);
    // Card dots take the creature's colour once its phenotype is known
    scope.effect(() => {
      phenotypeVersion.value;
      this.renderCards();
    });
    this.choose(0);
  }

  get selected(): CatalogueCreature {
    return this.creatures[this.index];
  }

  private choose(index: number) {
    this.index = index;
    const creature = this.creatures[index];
    this.name.textContent = creature.name;
    this.details.replaceChildren(
      "Diet Class: ", Object.assign(document.createElement("em"), { textContent: dietLabel(creature.carnivory) }),
      document.createElement("br"), `Fitness rating: ${creature.fitness.toFixed(2)}`,
      document.createElement("br"), `Origin: ${creature.source}`,
    );
    this.preview.setPhenotype(phenotypeCache.get(creature.genome) ?? null);
    void getPhenotype(creature.genome).then((p) => {
      if (p && this.selected === creature) this.preview.setPhenotype(p);
    });
    this.renderCards();
  }

  private renderCards() {
    this.deck.replaceChildren(...this.creatures.map((c, index) => {
      const pheno = phenotypeCache.get(c.genome);
      if (!pheno) void getPhenotype(c.genome);
      const color = badgeColor(pheno);
      const dot = Dot({ style: { background: color, boxShadow: `0 0 6px ${color}` } });
      const name = document.createElement("b");
      name.textContent = c.name;
      const card = Card({
        class: index === this.index ? "active" : "",
        children: [CardName({ children: [dot, name] }), CardMeta({ children: `${dietLabel(c.carnivory)} // F: ${c.fitness.toFixed(2)}` })],
      });
      card.addEventListener("click", () => this.choose(index));
      return card;
    }));
  }
}
