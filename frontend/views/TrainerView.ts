import { styled } from "../core/styled";
import { navigateTo } from "../core/router";
import { 
  IconWeights, IconBack, IconElite, IconTrophy, IconMutant, IconPlant 
} from "../components/Icons";
import { UnifiedDiagnosticsPanel } from "./UnifiedDiagnosticsPanel";

// --- STYLED COMPONENTS ---

const TrainerContainer = styled("div")`
  position: relative;
  width: 100vw;
  height: 100vh;
  background: radial-gradient(circle at center, #0f172a 0%, #020617 100%);
  color: #f1f5f9;
  font-family: monospace;
  box-sizing: border-box;
  overflow: hidden;
`;

const TrainerGrid = styled("main")`
  display: grid;
  grid-template-columns: 290px 1fr 350px;
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
  overflow-y: auto;
  overflow-x: hidden;
  height: 100%;
`;

const PanelHeader = styled("div")`
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  padding-bottom: 8px;
  margin-bottom: 12px;
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

const DropdownContainer = styled("div")`
  position: relative;
  margin-bottom: 8px;
`;

const DropdownButton = styled("button")`
  width: 100%;
  height: 32px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  padding: 2px 10px;
  background: rgba(15, 23, 42, 0.85);
  color: #00f2fe;
  font-weight: bold;
  font-family: monospace;
  border: 1px solid rgba(0, 242, 254, 0.3);
  border-radius: 4px;
  font-size: 0.7rem;
  outline: none;

  &:hover {
    background: rgba(0, 242, 254, 0.05);
  }
`;

const DropdownList = styled("div")`
  display: none;
  position: absolute;
  top: 35px;
  left: 0;
  right: 0;
  z-index: 100;
  max-height: 150px;
  box-shadow: 0 10px 25px rgba(0,0,0,0.6);
  background: rgba(15, 23, 42, 0.95);
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 4px;
  overflow-y: auto;
  padding: 4px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const InputRow = styled("div")`
  display: flex;
  gap: 6px;
`;

const TextInput = styled("input")`
  flex: 1;
  height: 30px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #10b981;
  font-family: monospace;
  font-size: 0.68rem;
  padding: 2px 8px;
  border-radius: 4px;
  outline: none;
  box-sizing: border-box;

  &:focus {
    border-color: #00f2fe;
  }
`;

const StatsCard = styled("div")`
  background: rgba(15, 23, 42, 0.45);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 10px;
`;

const StatsItem = styled("div")`
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.68rem;
`;

const StatsLabel = styled("span")`
  color: #94a3b8;
`;

const StatsValue = styled("span")`
  font-weight: bold;
  &.val-green { color: #10b981; }
  &.val-blue { color: #00f2fe; }
  &.val-gold { color: #f59e0b; }
  &.val-red { color: #ef4444; }
`;

const ButtonRow = styled("div")`
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
`;

const CyberButton = styled("button")`
  flex: 1;
  background: rgba(0, 242, 254, 0.05);
  border: 1px solid rgba(0, 242, 254, 0.2);
  color: #00f2fe;
  padding: 6px 4px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.7rem;
  cursor: pointer;
  transition: all 0.2s;
  font-weight: bold;

  &:hover {
    background: rgba(0, 242, 254, 0.15);
    border-color: #00f2fe;
    box-shadow: 0 0 10px rgba(0, 242, 254, 0.2);
  }

  &.btn-danger {
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.2);
    background: rgba(239, 68, 68, 0.05);

    &:hover {
      background: rgba(239, 68, 68, 0.15);
      border-color: #ef4444;
      box-shadow: 0 0 10px rgba(239, 68, 68, 0.2);
    }
  }
`;

const LegendCard = styled("div")`
  font-size: 0.55rem;
  color: #64748b;
  font-family: monospace;
  margin-bottom: 12px;
  border: 1px solid rgba(148, 163, 184, 0.1);
  border-radius: 4px;
  padding: 6px;
  background: rgba(15, 23, 42, 0.3);
  line-height: 1.4;
`;

const LegendGrid = styled("div")`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
  margin-top: 4px;
`;

const LegendItem = styled("div")`
  display: flex;
  align-items: center;
  gap: 4px;
`;

const PanelSection = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  padding-top: 12px;
  margin-bottom: 12px;
`;

const SliderControl = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const SliderLabelRow = styled("label")`
  display: flex;
  justify-content: space-between;
  font-size: 0.65rem;
  color: #94a3b8;
`;

const SliderInput = styled("input")`
  width: 100%;
  background: rgba(148, 163, 184, 0.1);
  height: 6px;
  border-radius: 3px;
  outline: none;
  cursor: pointer;
`;

const ModeSelect = styled("select")`
  width: 100%;
  background: rgba(15, 23, 42, 0.85) url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%2300f2fe" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>') no-repeat right 8px center;
  background-size: 14px;
  border: 1px solid rgba(0, 242, 254, 0.25);
  color: #00f2fe;
  padding: 8px 30px 8px 12px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.72rem;
  font-weight: bold;
  outline: none;
  cursor: pointer;
  margin-top: 4px;
  margin-bottom: 12px;
  text-shadow: 0 0 6px rgba(0, 242, 254, 0.35);
  box-shadow: inset 0 0 8px rgba(0, 242, 254, 0.05);
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);

  &:hover {
    border-color: #00f2fe;
    background-color: rgba(0, 242, 254, 0.05);
    box-shadow: 0 0 10px rgba(0, 242, 254, 0.15), inset 0 0 8px rgba(0, 242, 254, 0.05);
  }

  & option {
    background: #090d16;
    color: #f1f5f9;
    border: none;
    outline: none;
    font-size: 0.72rem;
  }
`;

const ExporterCard = styled("div")`
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  padding-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;

  h3 {
    font-size: 0.72rem;
    color: #94a3b8;
    margin: 0;
    text-transform: uppercase;
    display: flex;
    align-items: center;
  }

  p {
    font-size: 0.55rem;
    color: #475569;
    margin: 0;
    line-height: 1.35;
  }
`;

const GenomeTextArea = styled("textarea")`
  font-family: monospace;
  font-size: 0.55rem;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #10b981;
  width: 100%;
  height: 48px;
  padding: 6px;
  box-sizing: border-box;
  resize: none;
  word-break: break-all;
  outline: none;
  border-radius: 4px;
`;

const MiddleContainer = styled("div")`
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
`;

const HeaderNavRow = styled("div")`
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
`;

const SandboxGridContainer = styled("div")`
  flex: 1;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--tile-size, 130px), 1fr));
  grid-gap: 12px;
  overflow-y: auto;
  padding: 10px;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(14a, 165, 233, 0.08);
  border-radius: 6px;
  box-sizing: border-box;
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

// --- COMPONENT CREATOR ---

export function createTrainerView(): HTMLElement {
  const container = TrainerContainer({ id: "trainer-view-container" });

  // 1. Left Panel (Control Deck)
  const leftPanel = GlassPanel({
    children: [
      PanelHeader({
        children: [
          PanelTitle({ 
            children: [
              IconWeights,
              "Training Control"
            ] 
          }),
          PanelSubtitle({ children: "Generational Hyperparameters" })
        ]
      }),
      // Active Session Dropdown
      styled("div")``({
        style: { marginBottom: "12px", paddingBottom: "10px", borderBottom: "1px solid rgba(148, 163, 184, 0.1)" },
        children: [
          styled("label")``({
            style: { fontSize: "0.68rem", color: "#64748b", display: "block", marginBottom: "4px" },
            children: "Active Training Sessions"
          }),
          DropdownContainer({
            id: "training-dropdown",
            children: [
              DropdownButton({
                id: "training-dropdown-trigger",
                children: [
                  styled("span")``({ id: "training-dropdown-current", children: "default_run (New)" }),
                  styled("span")``({ style: { fontSize: "0.5rem", color: "#475569" }, children: "▼" })
                ]
              }),
              DropdownList({
                id: "training-list-container",
                style: { display: "none" }
              })
            ]
          }),
          InputRow({
            children: [
              TextInput({ id: "txt-new-training", type: "text", placeholder: "New Training Name..." }),
              styled("button")``({
                id: "btn-create-training",
                class: "btn btn-primary",
                style: {
                  background: "#00f2fe",
                  color: "#020617",
                  border: "none",
                  borderRadius: "4px",
                  fontSize: "0.65rem",
                  fontWeight: "bold",
                  cursor: "pointer",
                  padding: "0 10px",
                  height: "30px"
                },
                children: "Create"
              })
            ]
          })
        ]
      }),
      // Real-Time Metrics
      StatsCard({
        children: [
          StatsItem({
            children: [
              StatsLabel({ children: "Generation:" }),
              StatsValue({ id: "stat-gen", class: "val-blue", children: "1" })
            ]
          }),
          StatsItem({
            children: [
              StatsLabel({ children: "Best Fitness:" }),
              StatsValue({ id: "stat-best-fit", class: "val-green", children: "0.0" })
            ]
          }),
          StatsItem({
            children: [
              StatsLabel({ children: "Avg Fitness:" }),
              StatsValue({ id: "stat-avg-fit", class: "val-gold", children: "0.0" })
            ]
          }),
          StatsItem({
            children: [
              StatsLabel({ children: "Time Remaining:" }),
              StatsValue({ id: "stat-timer", class: "val-red", children: "5.0s" })
            ]
          })
        ]
      }),
      ButtonRow({
        children: [
          CyberButton({ id: "btn-start", children: "Start Training" }),
          CyberButton({ id: "btn-reset-train", class: "btn-danger", children: "Reset Evolution" })
        ]
      }),
      // Population Origin Key
      LegendCard({
        children: [
          styled("b")``({ style: { color: "#94a3b8", display: "block", marginBottom: "6px" }, children: "POPULATION ORIGINS:" }),
          LegendGrid({
            children: [
              LegendItem({ children: [IconElite, `<span style='color:#fbbf24;'>Elite Winner</span>`] }),
              LegendItem({ children: [IconTrophy, `<span style='color:#60a5fa;'>Hall of Fame</span>`] }),
              LegendItem({ children: [IconMutant, `<span style='color:#c084fc;'>Mutant Clone</span>`] }),
              LegendItem({ children: [IconPlant, `<span style='color:#34d399;'>Fresh Random</span>`] })
            ]
          })
        ]
      }),
      // Tuning Sliders
      PanelSection({
        children: [
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Grid Size (N Sandboxes)" }),
                  styled("span")``({ id: "lbl-grid-size", children: "16" })
                ]
              }),
              SliderInput({ id: "slider-grid-size", type: "range", min: "4", max: "100", value: "16", step: "1" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Warp Speedup" }),
                  styled("span")``({ id: "lbl-speedup", children: "1x" })
                ]
              }),
              SliderInput({ id: "slider-speedup", type: "range", min: "1", max: "12", value: "1", step: "1" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Training Mode" }),
                  styled("span")``({ id: "lbl-trainer-mode", children: "Standard" })
                ]
              }),
              ModeSelect({
                id: "select-trainer-mode",
                children: [
                  styled("option")``({ value: "standard", children: "Standard Chamber (1000x1000)" }),
                  styled("option")``({ value: "exploration", children: "Exploration (2500x2500)" })
                ]
              })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Laboratory Zoom (Tile Size)" }),
                  styled("span")``({ id: "lbl-zoom-tiles", children: "140px" })
                ]
              }),
              SliderInput({ id: "slider-zoom-tiles", type: "range", min: "100", max: "400", value: "140", step: "5" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Elite Ratio (M %)" }),
                  styled("span")``({ id: "lbl-elite-ratio", children: "25%" })
                ]
              }),
              SliderInput({ id: "slider-elite-ratio", type: "range", min: "5", max: "100", value: "25", step: "1" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Genome Mutation Rate" }),
                  styled("span")``({ id: "lbl-mutation-rate", children: "15%" })
                ]
              }),
              SliderInput({ id: "slider-mutation-rate", type: "range", min: "0", max: "50", value: "15", step: "1" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Random Inflow (N %)" }),
                  styled("span")``({ id: "lbl-inflow-rate", children: "10%" })
                ]
              }),
              SliderInput({ id: "slider-inflow-rate", type: "range", min: "0", max: "100", value: "10", step: "1" })
            ]
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Hall of Fame (%)" }),
                  styled("span")``({ id: "lbl-hof-rate", children: "10%" })
                ]
              }),
              SliderInput({ id: "slider-hof-rate", type: "range", min: "0", max: "100", value: "10", step: "1" })
            ]
          }),
          // Multi-trial Checkbox
          styled("div")``({
            style: { display: "flex", alignItems: "center", gap: "8px", marginTop: "5px" },
            children: [
              styled("input")``({ id: "chk-multi-trial", type: "checkbox", style: { cursor: "pointer" } }),
              styled("label")``({
                htmlFor: "chk-multi-trial",
                style: { fontSize: "0.65rem", cursor: "pointer", userSelect: "none" },
                children: "Enable Multi-Trial (3 Runs per Gen)"
              })
            ]
          }),
          // Lamarckian Checkbox
          styled("div")``({
            style: { display: "flex", alignItems: "center", gap: "8px", marginTop: "5px" },
            children: [
              styled("input")``({ id: "chk-lamarck-syn", type: "checkbox", style: { cursor: "pointer" } }),
              styled("label")``({
                htmlFor: "chk-lamarck-syn",
                style: { fontSize: "0.65rem", cursor: "pointer", userSelect: "none" },
                children: "Lamarckian Synapse Inheritance"
              })
            ]
          })
        ]
      }),
      // Champion Exporter
      ExporterCard({
        children: [
          styled("h3")``({ 
            children: [
              IconTrophy,
              "Champion Exporter (Local Cache)"
            ] 
          }),
          styled("p")``({ children: "Copy-paste this DNA into config.json's rules.progenitorGenome to boost live ocean spawn rates." }),
          GenomeTextArea({ id: "txt-dna", readonly: true, placeholder: "No champion evolved yet. Start training..." }),
          styled("button")``({
            id: "btn-copy-dna",
            style: {
              background: "#10b981",
              border: "none",
              borderRadius: "4px",
              color: "#020617",
              fontFamily: "monospace",
              fontSize: "0.7rem",
              fontWeight: "bold",
              cursor: "pointer",
              padding: "8px 12px",
              marginTop: "4px"
            },
            children: "Copy Champion DNA"
          })
        ]
      })
    ]
  });

  // 2. Middle Column: Header Row + Sandbox Chambers Grid
  const middleCol = MiddleContainer({
    children: [
      HeaderNavRow({
        children: [
          styled("h2")``({ style: { margin: 0, fontSize: "0.85rem", color: "#fff", textTransform: "uppercase" }, children: " Evolutionary Sandboxes" }),
          BackToMenuButton({
            onClick: () => navigateTo("main-menu"),
            children: [
              IconBack,
              "Main Menu"
            ]
          })
        ]
      }),
      SandboxGridContainer({ id: "sandbox-grid" })
    ]
  });

  // Reusable, reactive, and completely encapsulated Diagnostics Panel!
  const rightPanel = GlassPanel({
    id: "inspect-panel",
    class: "panel-right"
  });
  const diagnosticsPanel = new UnifiedDiagnosticsPanel("trainer");
  rightPanel.appendChild(diagnosticsPanel.element);

  const grid = TrainerGrid({
    children: [
      leftPanel,
      middleCol,
      rightPanel
    ]
  });

  container.appendChild(grid);

  return container;
}
