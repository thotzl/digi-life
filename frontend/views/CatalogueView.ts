import { styled } from "../core/styled";
import { navigateTo, currentView } from "../core/router";
import { effect } from "@preact/signals-core";
import { safeInvoke } from "../api";
import { IconCatalogue, IconBack } from "../components/Icons";
import { UnifiedDiagnosticsPanel } from "./UnifiedDiagnosticsPanel";
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAdrenaline, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype
} from "../signals";

// --- STYLED COMPONENTS ---

const CatalogueContainer = styled("div")`
  position: relative;
  width: 100vw;
  height: 100vh;
  background: radial-gradient(circle at center, #0b1528 0%, #020617 100%);
  color: #f1f5f9;
  font-family: monospace;
  box-sizing: border-box;
  overflow: hidden;
`;

const CatalogueGrid = styled("main")`
  display: grid;
  grid-template-columns: 1fr 360px;
  grid-gap: 15px;
  padding: 15px;
  box-sizing: border-box;
  height: 100%;
  width: 100%;
`;

const GlassPanel = styled("section")`
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(148, 163, 184, 0.12);
  border-radius: 8px;
  padding: 15px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
`;

const PanelHeader = styled("div")`
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  padding-bottom: 8px;
  margin-bottom: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const PanelTitle = styled("h2")`
  font-size: 0.85rem;
  font-weight: bold;
  letter-spacing: 1px;
  color: #fff;
  margin: 0;
  text-transform: uppercase;
  display: flex;
  align-items: center;
`;

const PanelSubtitle = styled("span")`
  font-size: 0.6rem;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  margin-top: 2px;
  display: block;
`;

const BackToMenuButton = styled("button")`
  background: rgba(15, 23, 42, 0.7);
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 4px;
  color: #94a3b8;
  padding: 4px 10px;
  font-family: monospace;
  font-size: 0.62rem;
  cursor: pointer;
  transition: all 0.2s;
  letter-spacing: 1px;
  text-transform: uppercase;
  display: flex;
  align-items: center;

  &:hover {
    color: #fff;
    border-color: #00f2fe;
    box-shadow: 0 0 10px rgba(0, 242, 254, 0.25);
  }
`;

const ScrollableList = styled("div")`
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;

  &::-webkit-scrollbar {
    width: 4px;
  }
  &::-webkit-scrollbar-track {
    background: transparent;
  }
  &::-webkit-scrollbar-thumb {
    background: rgba(148, 163, 184, 0.15);
    border-radius: 2px;
  }
`;

const SpecimenCard = styled("div")`
  background: rgba(30, 41, 59, 0.15);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 8px 12px;
  cursor: pointer;
  transition: all 0.18s ease-in-out;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  position: relative;
  overflow: hidden;

  &:hover {
    background: rgba(15, 23, 42, 0.35);
    border-color: #f59e0b;
    box-shadow: 0 0 10px rgba(245, 158, 11, 0.04);
  }

  &.active {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.04);
    box-shadow: 0 0 12px rgba(245, 158, 11, 0.06);
  }
`;

const SpecimenGlowDot = styled("div")`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  box-shadow: 0 0 8px currentColor;
  flex-shrink: 0;
`;

const SpecimenCardHeader = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 160px;
  flex-shrink: 0;

  h3 {
    margin: 0;
    font-size: 0.8rem;
    color: #fff;
    font-weight: bold;
  }

  span.source {
    font-size: 0.55rem;
    color: #64748b;
    text-transform: uppercase;
  }
`;

const SpecimenMetricsRow = styled("div")`
  display: flex;
  align-items: center;
  gap: 24px;
  font-size: 0.65rem;
  color: #94a3b8;
  flex: 1;
`;

const CardButtonGroup = styled("div")`
  display: flex;
  gap: 6px;
  flex-shrink: 0;
`;

const SmallCardButton = styled("button")`
  flex: 1;
  background: rgba(148, 163, 184, 0.04);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #94a3b8;
  border-radius: 4px;
  padding: 4px;
  font-family: monospace;
  font-size: 0.58rem;
  cursor: pointer;
  transition: all 0.15s;
  font-weight: bold;

  &:hover {
    background: rgba(245, 158, 11, 0.08);
    border-color: #f59e0b;
    color: #fff;
  }

  &.btn-spawn {
    background: rgba(16, 185, 129, 0.05);
    border-color: rgba(16, 185, 129, 0.2);
    color: #10b981;

    &:hover {
      background: rgba(16, 185, 129, 0.15);
      border-color: #10b981;
      color: #fff;
      box-shadow: 0 0 10px rgba(16, 185, 129, 0.2);
    }
  }

  &.btn-delete {
    background: rgba(239, 68, 68, 0.03);
    border-color: rgba(239, 68, 68, 0.15);
    color: #ef4444;

    &:hover {
      background: rgba(239, 68, 68, 0.12);
      border-color: #ef4444;
      color: #fff;
      box-shadow: 0 0 10px rgba(239, 68, 68, 0.2);
    }
  }
`;

// Modal backdrop and contents
const CyberModalBackdrop = styled("div")`
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(2, 6, 23, 0.85);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(8px);
`;

const CyberModalBox = styled("div")`
  background: rgba(15, 23, 42, 0.9);
  border: 1px solid rgba(0, 242, 254, 0.3);
  border-radius: 8px;
  width: 360px;
  padding: 24px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 0 30px rgba(0, 242, 254, 0.15);
  animation: slide-fade-in 0.2s ease-out;

  h3 {
    margin: 0;
    font-size: 1rem;
    color: #fff;
    text-transform: uppercase;
    letter-spacing: 1px;
    border-bottom: 1px solid rgba(148, 163, 184, 0.1);
    padding-bottom: 8px;
  }

  p {
    margin: 0;
    font-size: 0.68rem;
    color: #94a3b8;
    line-height: 1.45;
  }
`;

// --- MAIN CLASS COMPONENT ---

export class CatalogueView {
  public element: HTMLElement;
  private listEl: HTMLElement;
  private diagnosticsPanel: UnifiedDiagnosticsPanel;
  private modalContainer: HTMLElement;

  constructor() {
    this.element = CatalogueContainer({ id: "catalogue-view-container" });

    // Assemble Header row
    const header = PanelHeader({
      children: [
        styled("div")``({
          children: [
            PanelTitle({ children: [IconCatalogue, "Creature Catalogue"] }),
            PanelSubtitle({ children: "Persistent Cryo-Preservation Ledger" })
          ]
        }),
        BackToMenuButton({
          onClick: () => {
            selectedId.value = null; // Reset selection
            navigateTo("main-menu");
          },
          children: [
            IconBack,
            "Main Menu"
          ]
        })
      ]
    });

    this.listEl = ScrollableList();
    this.modalContainer = document.createElement("div");

    const leftPanel = GlassPanel({
      children: [
        header,
        this.listEl
      ]
    });

    const rightPanel = GlassPanel({
      class: "panel-right"
    });

    // Instantiate modular UnifiedDiagnosticsPanel
    this.diagnosticsPanel = new UnifiedDiagnosticsPanel("catalogue");
    rightPanel.appendChild(this.diagnosticsPanel.element);

    const grid = CatalogueGrid({
      children: [
        leftPanel,
        rightPanel
      ]
    });

    this.element.appendChild(grid);
    this.element.appendChild(this.modalContainer);

    // Initial update list
    this.refreshList();

    // Listen to currentView changes to auto-refresh whenever catalog is opened
    effect(() => {
      if (currentView.value === "catalogue") {
        this.refreshList();
      }
    });
  }

  /**
   * Queries SQLite and populates the scrollable list of saved creatures.
   */
  public async refreshList() {
    this.listEl.innerHTML = `<div class="fallback-state">Sensing database ledger...</div>`;
    
    try {
      const creatures = await safeInvoke("get_catalogue_creatures");
      if (!creatures || creatures.length === 0) {
        this.listEl.innerHTML = `
          <div class="fallback-state">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity:0.3; margin-bottom:10px;"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
            <p>Your cryo-preservation vault is empty.<br/>Save elites from the Training Chamber or specimens from the Ocean.</p>
          </div>
        `;
        return;
      }

      this.listEl.innerHTML = "";

      creatures.forEach((creature: any) => {
        // Compile the phenotype color for the glow badge
        const charVal = creature.genome.charCodeAt(17) || 65; // Use color locus
        const primaryH = (charVal - 65) * 13.8;
        const colorStyle = `color: hsla(${primaryH}, 85%, 55%, 1);`;

        let dietLabel = "Omnivore";
        if (creature.carnivory >= 0.55) dietLabel = "Predator";
        else if (creature.carnivory < 0.3) dietLabel = "Herbivore";

        const card = SpecimenCard({
          class: selectedId.value === creature.id ? "active" : "",
          onClick: () => this.selectCreature(creature)
        });

        const glowDot = SpecimenGlowDot({ style: colorStyle });

        const headerInfo = SpecimenCardHeader({
          children: [
            styled("h3")``({ children: creature.name }),
            styled("span")``({ class: "source", children: creature.source })
          ]
        });

        const metrics = SpecimenMetricsRow({
          children: [
            styled("span")``({ children: `Diet: ${dietLabel}` }),
            styled("span")``({ children: `Fit: ${creature.fitness.toFixed(0)}` })
          ]
        });

        const btnSpawn = SmallCardButton({
          class: "btn-spawn",
          onClick: (e: Event) => {
            e.stopPropagation();
            this.openSpawnDialog(creature);
          },
          children: "Release Clone"
        });

        const btnTrain = SmallCardButton({
          style: { borderColor: "rgba(59, 130, 246, 0.3)", color: "#3b82f6", background: "rgba(59, 130, 246, 0.04)" },
          onClick: (e: Event) => {
            e.stopPropagation();
            this.openTrainDialog(creature);
          },
          children: "Seed Training"
        });

        const btnRename = SmallCardButton({
          onClick: (e: Event) => {
            e.stopPropagation();
            this.renameCreature(creature);
          },
          children: "Rename"
        });

        const btnDelete = SmallCardButton({
          class: "btn-delete",
          onClick: (e: Event) => {
            e.stopPropagation();
            this.deleteCreature(creature);
          },
          children: "Wipe"
        });

        const buttons = CardButtonGroup({
          children: [
            btnSpawn,
            btnTrain,
            btnRename,
            btnDelete
          ]
        });

        card.appendChild(glowDot);
        card.appendChild(headerInfo);
        card.appendChild(metrics);
        card.appendChild(buttons);

        this.listEl.appendChild(card);
      });
    } catch (e) {
      console.error("[Catalogue] Failed to load ledger list:", e);
      this.listEl.innerHTML = `<div class="fallback-state" style="color:#ef4444;">Failed to read SQLite database: ${e}</div>`;
    }
  }

  /**
   * Sets the global Signals to focused creature values, driving the UnifiedDiagnosticsPanel reactively.
   */
  private async selectCreature(creature: any) {
    selectedId.value = creature.id;
    selectedName.value = creature.name;
    selectedTaxa.value = `Provenance: ${creature.source}`;
    selectedStatus.value = "Cryo-Frozen";
    selectedEnergy.value = 100;
    selectedMaxEnergy.value = 100;
    selectedAdrenaline.value = 1.0;
    selectedAge.value = 0;
    selectedGenome.value = creature.genome;
    selectedMethylations.value = creature.methylations;

    // Fetch full compiled phenotype from the backend
    try {
      const pheno = await safeInvoke("get_fossil_phenotype", { genome: creature.genome });
      if (pheno) {
        selectedPhenotype.value = pheno;
      }
    } catch (e) {
      console.error("[Catalogue] Failed to decompile phenotype:", e);
    }

    // Highlighting selected card
    document.querySelectorAll(`.${this.listEl.className} > div`).forEach(el => el.classList.remove("active"));
    this.refreshList(); // Re-render highlights cleanly
  }

  /**
   * Opens the custom Cyber-Modal dialog for spawning the creature with synaptic toggle.
   */
  private openSpawnDialog(creature: any) {
    this.modalContainer.innerHTML = "";

    const backdrop = CyberModalBackdrop();
    
    const checkbox = styled("input")``({
      id: "chk-load-syn",
      type: "checkbox",
      checked: false,
      style: { cursor: "pointer", width: "16px", height: "16px" }
    });

    const labelRow = styled("label")``({
      htmlFor: "chk-load-syn",
      style: { display: "flex", alignItems: "center", gap: "8px", fontSize: "0.7rem", cursor: "pointer", userSelect: "none" },
      children: [
        checkbox,
        styled("span")``({ children: "Load with learned synaptic weights (imprinting)" })
      ]
    });

    const btnConfirm = styled("button")``({
      style: {
        background: "#10b981",
        color: "#020617",
        border: "none",
        borderRadius: "4px",
        fontFamily: "monospace",
        fontWeight: "bold",
        fontSize: "0.7rem",
        cursor: "pointer",
        padding: "8px 16px"
      },
      onClick: async () => {
        const loadSyn = checkbox.checked;
        try {
          const ok = await safeInvoke("spawn_catalogue_creature_to_ocean", {
            id: creature.id,
            loadLearnedSynapses: loadSyn
          });
          if (ok) {
            alert(`Clone of "${creature.name}" was successfully released into the ocean!`);
          }
        } catch (err) {
          alert(`Error during spawning: ${err}`);
        } finally {
          this.modalContainer.innerHTML = ""; // close modal
        }
      },
      children: "Release Clone"
    });

    const btnCancel = styled("button")``({
      style: {
        background: "rgba(148, 163, 184, 0.08)",
        border: "1px solid rgba(148, 163, 184, 0.15)",
        color: "#94a3b8",
        borderRadius: "4px",
        fontFamily: "monospace",
        fontSize: "0.7rem",
        cursor: "pointer",
        padding: "8px 16px"
      },
      onClick: () => {
        this.modalContainer.innerHTML = ""; // close modal
      },
      children: "Cancel"
    });

    const buttonsRow = styled("div")``({
      style: { display: "flex", justifyContent: "flex-end", gap: "8px", borderTop: "1px solid rgba(148, 163, 184, 0.1)", paddingTop: "12px", marginTop: "4px" },
      children: [
        btnCancel,
        btnConfirm
      ]
    });

    const modalBox = CyberModalBox({
      children: [
        styled("h3")``({ children: "🚀 Release Clone" }),
        styled("p")``({ children: `Should "${creature.name}" be released into the active ocean as an exact clone (including learned brain reflexes) or as a new genetic seed?` }),
        labelRow,
        buttonsRow
      ]
    });

    backdrop.appendChild(modalBox);
    this.modalContainer.appendChild(backdrop);
  }

  /**
   * Opens the custom Cyber-Modal dialog for seeding the creature into an existing training run.
   */
  private async openTrainDialog(creature: any) {
    this.modalContainer.innerHTML = "";

    try {
      const runs = await safeInvoke("get_trainer_runs");
      if (!runs || runs.length === 0) {
        alert("No training runs found in the database. First create a run in the Training Chamber!");
        return;
      }

      const backdrop = CyberModalBackdrop();

      const runSelect = styled("select")``({
        style: {
          width: "100%",
          background: "rgba(15, 23, 42, 0.85)",
          color: "#00f2fe",
          fontFamily: "monospace",
          fontSize: "0.75rem",
          fontWeight: "bold",
          border: "1px solid rgba(0, 242, 254, 0.3)",
          borderRadius: "4px",
          padding: "8px",
          outline: "none"
        }
      });

      // Populate select options with all available runs
      runs.forEach((run: any) => {
        const opt = document.createElement("option");
        opt.value = JSON.stringify({ id: run.run_id, gen: run.max_gen });
        opt.innerText = `${run.run_id} (Generation ${run.max_gen}, Best Fit: ${run.max_fit.toFixed(0)})`;
        runSelect.appendChild(opt);
      });

      const btnConfirm = styled("button")``({
        style: {
          background: "#3b82f6",
          color: "#020617",
          border: "none",
          borderRadius: "4px",
          fontFamily: "monospace",
          fontWeight: "bold",
          fontSize: "0.7rem",
          cursor: "pointer",
          padding: "8px 16px"
        },
        onClick: async () => {
          const selectedData = JSON.parse(runSelect.value);
          try {
            const ok = await safeInvoke("add_catalogue_creature_to_training", {
              creatureId: creature.id,
              runId: selectedData.id,
              generation: selectedData.gen
            });
            if (ok) {
              alert(`Creature "${creature.name}" successfully injected into training run "${selectedData.id}" (Generation ${selectedData.gen})!\n\nIt will compete in one of the sandbox chambers starting from the next generation.`);
            }
          } catch (err) {
            alert(`Error injecting into training: ${err}`);
          } finally {
            this.modalContainer.innerHTML = ""; // close modal
          }
        },
        children: "Confirm Injection"
      });

      const btnCancel = styled("button")``({
        style: {
          background: "rgba(148, 163, 184, 0.08)",
          border: "1px solid rgba(148, 163, 184, 0.15)",
          color: "#94a3b8",
          borderRadius: "4px",
          fontFamily: "monospace",
          fontSize: "0.7rem",
          cursor: "pointer",
          padding: "8px 16px"
        },
        onClick: () => {
          this.modalContainer.innerHTML = ""; // close modal
        },
        children: "Cancel"
      });

      const buttonsRow = styled("div")``({
        style: { display: "flex", justifyContent: "flex-end", gap: "8px", borderTop: "1px solid rgba(148, 163, 184, 0.1)", paddingTop: "12px", marginTop: "12px" },
        children: [
          btnCancel,
          btnConfirm
        ]
      });

      const modalBox = CyberModalBox({
        children: [
          styled("h3")``({ children: "🏋️ Inject into Training" }),
          styled("p")``({ style: { marginBottom: "12px" }, children: `Select which of your persistent training runs you want to inject "${creature.name}" into:` }),
          runSelect,
          buttonsRow
        ]
      });

      backdrop.appendChild(modalBox);
      this.modalContainer.appendChild(backdrop);
    } catch (e) {
      console.error("[Catalogue] Failed to fetch runs for inject dialog:", e);
    }
  }

  /**
   * Prompts to rename a saved creature.
   */
  private async renameCreature(creature: any) {
    const newName = prompt(`Enter a new name for "${creature.name}":`, creature.name);
    if (newName === null) return;
    const name = newName.trim();
    if (!name) return;

    try {
      const ok = await safeInvoke("rename_catalogue_creature", { id: creature.id, newName: name });
      if (ok) {
        if (selectedId.value === creature.id) {
          selectedName.value = name;
        }
        this.refreshList();
      }
    } catch (e) {
      alert(`Error renaming: ${e}`);
    }
  }

  /**
   * Confirms and deletes a saved creature.
   */
  private async deleteCreature(creature: any) {
    const confirmWipe = confirm(`Are you sure you want to permanently delete species "${creature.name}" from the catalogue?`);
    if (!confirmWipe) return;

    try {
      const ok = await safeInvoke("delete_from_catalogue", { id: creature.id });
      if (ok) {
        if (selectedId.value === creature.id) {
          selectedId.value = null; // Clear focus
        }
        this.refreshList();
      }
    } catch (e) {
      alert(`Error deleting: ${e}`);
    }
  }
}
export function createCatalogueView(): HTMLElement {
  const catalogue = new CatalogueView();
  return catalogue.element;
}
