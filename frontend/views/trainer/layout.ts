// Training Room DOM (TCK-160): styled components and the layout builder returning element references.
import { styled } from "../../core/styled";
import { WARP_SLIDER_MAX } from "../../shared/warp";
import { IconBack, IconElite, IconMutant, IconPlant, IconTrophy, IconWeights } from "../../components/Icons";

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

const ProgressCard = styled("div")`
  background: rgba(15, 23, 42, 0.45);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 8px;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const ProgressCanvas = styled("canvas")`
  width: 100%;
  height: 90px;
  display: block;
`;

const ProgressLegend = styled("div")`
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 0.55rem;
  color: #64748b;
`;

const PendingNote = styled("span")`
  font-size: 0.58rem;
  color: #f59e0b;
  min-height: 0.7rem;
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
  border: 1px solid rgba(14, 165, 233, 0.08);
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

/// --- LAYOUT ---

/** Element references of the trainer view (built once; the controller never looks elements up by id). */
export interface TrainerLayout {
  root: HTMLElement;
  btnBack: HTMLButtonElement;
  runTrigger: HTMLButtonElement;
  runCurrent: HTMLElement;
  runList: HTMLElement;
  txtNewRun: HTMLInputElement;
  btnCreateRun: HTMLButtonElement;
  statGen: HTMLElement;
  statBestFit: HTMLElement;
  statAvgFit: HTMLElement;
  statSuccess: HTMLElement;
  statMedianFit: HTMLElement;
  statFirstCatch: HTMLElement;
  statLevel: HTMLElement;
  statTimer: HTMLElement;
  canvasProgress: HTMLCanvasElement;
  btnStart: HTMLButtonElement;
  btnReset: HTMLButtonElement;
  sliderPopulation: HTMLInputElement;
  lblPopulation: HTMLElement;
  sliderGridSize: HTMLInputElement;
  lblGridSize: HTMLElement;
  sliderSpeedup: HTMLInputElement;
  lblSpeedup: HTMLElement;
  selectMode: HTMLSelectElement;
  lblMode: HTMLElement;
  lblPendingSwitch: HTMLElement;
  sliderLevel: HTMLInputElement;
  lblLevel: HTMLElement;
  sliderZoomTiles: HTMLInputElement;
  lblZoomTiles: HTMLElement;
  sliderEliteRatio: HTMLInputElement;
  lblEliteRatio: HTMLElement;
  sliderMutation: HTMLInputElement;
  lblMutation: HTMLElement;
  sliderInflow: HTMLInputElement;
  lblInflow: HTMLElement;
  sliderHof: HTMLInputElement;
  lblHof: HTMLElement;
  chkMultiTrial: HTMLInputElement;
  chkLamarck: HTMLInputElement;
  txtDna: HTMLTextAreaElement;
  btnCopyDna: HTMLButtonElement;
  grid: HTMLElement;
  inspector: HTMLElement;
}

const Div = styled("div")``;
const Span = styled("span")``;
const B = styled("b")``;
const H2 = styled("h2")``;
const H3 = styled("h3")``;
const P = styled("p")``;
const Label = styled("label")``;
const Option = styled("option")``;
const Input = styled("input")``;
const Button = styled("button")``;
const CheckRow = styled("div")`
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 5px;
`;

function slider(id: string, labelId: string, title: string, label: string, attrs: { min: string; max: string; value: string; step: string }) {
  const input = SliderInput({ id, type: "range", ...attrs }) as HTMLInputElement;
  const lbl = Span({ id: labelId, children: label });
  return { input, lbl, element: SliderControl({ children: [SliderLabelRow({ children: [Span({ children: title }), lbl] }), input] }) };
}

function checkbox(id: string, text: string) {
  const input = Input({ id, type: "checkbox", style: { cursor: "pointer" } }) as HTMLInputElement;
  const element = CheckRow({
    children: [input, Label({ htmlFor: id, style: { fontSize: "0.65rem", cursor: "pointer", userSelect: "none" }, children: text })],
  });
  return { input, element };
}

function stat(id: string, cls: string, label: string, value: string) {
  const el = StatsValue({ id, class: cls, children: value });
  return { el, row: StatsItem({ children: [StatsLabel({ children: label }), el] }) };
}

/** Builds the trainer DOM. Ids are kept for styles.css and the UI check scripts. */
export function buildTrainerLayout(): TrainerLayout {
  const root = TrainerContainer({ id: "trainer-view-container" });

  const runCurrent = Span({ id: "training-dropdown-current", children: "default_run (New)" });
  const runTrigger = DropdownButton({ id: "training-dropdown-trigger", children: [runCurrent, Span({ style: { fontSize: "0.5rem", color: "#475569" }, children: "▼" })] });
  const runList = DropdownList({ id: "training-list-container", style: { display: "none" } });
  const txtNewRun = TextInput({ id: "txt-new-training", type: "text", placeholder: "New Training Name..." });
  const btnCreateRun = Button({
    id: "btn-create-training",
    class: "btn btn-primary",
    style: { background: "#00f2fe", color: "#020617", border: "none", borderRadius: "4px", fontSize: "0.65rem", fontWeight: "bold", cursor: "pointer", padding: "0 10px", height: "30px" },
    children: "Create",
  });

  const gen = stat("stat-gen", "val-blue", "Generation:", "1");
  const best = stat("stat-best-fit", "val-green", "Best Fitness:", "0.00");
  const avg = stat("stat-avg-fit", "val-gold", "Avg Fitness:", "0.00");
  const success = stat("stat-success", "val-green", "Success Rate:", "–");
  const median = stat("stat-median-fit", "val-blue", "Median Fitness:", "–");
  const firstCatch = stat("stat-first-catch", "val-gold", "First Catch (median):", "–");
  const level = stat("stat-level", "val-blue", "Difficulty Level:", "0 % (fixed)");
  const timer = stat("stat-timer", "val-red", "Time Remaining:", "5.0s");

  const canvasProgress = ProgressCanvas({ id: "canvas-progress", width: "520", height: "180" });
  const btnStart = CyberButton({ id: "btn-start", children: "Start Training" });
  const btnReset = CyberButton({ id: "btn-reset-train", class: "btn-danger", children: "Reset Evolution" });

  // Population: value = index into POPULATION_STEPS (metrics.ts); selection, breeding and stats run over the whole
  // population, the grid shows the first N sandboxes (TCK-192)
  const population = slider("slider-population", "lbl-population", "Population", "100", { min: "0", max: "4", value: "2", step: "1" });
  const gridSize = slider("slider-grid-size", "lbl-grid-size", "Visible Sandboxes", "16", { min: "4", max: "100", value: "16", step: "1" });
  const speedup = slider("slider-speedup", "lbl-speedup", "Warp Speedup", "1x", { min: "1", max: String(WARP_SLIDER_MAX), value: "1", step: "1" });
  const lblMode = Span({ id: "lbl-trainer-mode", children: "Standard" });
  const selectMode = ModeSelect({
    id: "select-trainer-mode",
    children: [
      Option({ value: "standard", children: "Standard Chamber (1000x1000)" }),
      Option({ value: "exploration", children: "Exploration (3500x3500)" }),
      Option({ value: "curriculum", children: "Curriculum (auto)" }),
      Option({ value: "manual", children: "Manual Level (slider)" }),
    ],
  });
  const lblPendingSwitch = PendingNote({ id: "lbl-pending-switch" });
  // Difficulty level 0 = small room .. 100 % = large room; moving it sets a manual level (TCK-176)
  const levelSlider = slider("slider-level", "lbl-level", "Difficulty Level (manual)", "0%", { min: "0", max: "100", value: "0", step: "5" });
  const zoom = slider("slider-zoom-tiles", "lbl-zoom-tiles", "Laboratory Zoom (Tile Size)", "140px", { min: "100", max: "400", value: "140", step: "5" });
  const elite = slider("slider-elite-ratio", "lbl-elite-ratio", "Elite Ratio (M %)", "25%", { min: "5", max: "100", value: "25", step: "1" });
  const mutation = slider("slider-mutation-rate", "lbl-mutation-rate", "Mutation Ops per Child", "1–3", { min: "1", max: "8", value: "3", step: "1" });
  const inflow = slider("slider-inflow-rate", "lbl-inflow-rate", "Random Inflow (N %)", "10%", { min: "0", max: "100", value: "10", step: "1" });
  const hof = slider("slider-hof-rate", "lbl-hof-rate", "Hall of Fame (%)", "10%", { min: "0", max: "100", value: "10", step: "1" });
  const multiTrial = checkbox("chk-multi-trial", "Enable Multi-Trial (several layouts per generation)");
  const lamarck = checkbox("chk-lamarck-syn", "Lamarckian Synapse Inheritance");

  const txtDna = GenomeTextArea({ id: "txt-dna", readonly: true, placeholder: "No champion evolved yet. Start training..." });
  const btnCopyDna = Button({
    id: "btn-copy-dna",
    style: { background: "#10b981", border: "none", borderRadius: "4px", color: "#020617", fontFamily: "monospace", fontSize: "0.7rem", fontWeight: "bold", cursor: "pointer", padding: "8px 12px", marginTop: "4px" },
    children: "Copy Champion DNA",
  });

  const leftPanel = GlassPanel({
    children: [
      PanelHeader({ children: [PanelTitle({ children: [IconWeights, "Training Control"] }), PanelSubtitle({ children: "Generational Hyperparameters" })] }),
      Div({
        style: { marginBottom: "12px", paddingBottom: "10px", borderBottom: "1px solid rgba(148, 163, 184, 0.1)" },
        children: [
          Label({ style: { fontSize: "0.68rem", color: "#64748b", display: "block", marginBottom: "4px" }, children: "Active Training Sessions" }),
          DropdownContainer({ id: "training-dropdown", children: [runTrigger, runList] }),
          InputRow({ children: [txtNewRun, btnCreateRun] }),
        ],
      }),
      StatsCard({ children: [gen.row, best.row, avg.row, success.row, median.row, firstCatch.row, level.row, timer.row] }),
      // Progress over the generations of the current run (TCK-177)
      ProgressCard({
        children: [
          StatsLabel({ style: { fontSize: "0.6rem" }, children: "Progress (current run)" }),
          canvasProgress,
          ProgressLegend({
            children: [
              Span({ style: { color: "#10b981" }, children: "■ success %" }),
              Span({ style: { color: "#f59e0b" }, children: "■ best fit" }),
              Span({ style: { color: "#00f2fe" }, children: "■ median fit" }),
              Span({ style: { color: "#c084fc" }, children: "■ level %" }),
            ],
          }),
        ],
      }),
      ButtonRow({ children: [btnStart, btnReset] }),
      LegendCard({
        children: [
          B({ style: { color: "#94a3b8", display: "block", marginBottom: "6px" }, children: "POPULATION ORIGINS:" }),
          LegendGrid({
            children: [
              LegendItem({ children: [IconElite, Span({ style: { color: "#fbbf24" }, children: "Elite Winner" })] }),
              LegendItem({ children: [IconTrophy, Span({ style: { color: "#60a5fa" }, children: "Hall of Fame" })] }),
              LegendItem({ children: [IconMutant, Span({ style: { color: "#c084fc" }, children: "Mutant Clone" })] }),
              LegendItem({ children: [IconPlant, Span({ style: { color: "#34d399" }, children: "Fresh Random" })] }),
            ],
          }),
        ],
      }),
      PanelSection({
        children: [
          population.element,
          gridSize.element,
          speedup.element,
          SliderControl({ children: [SliderLabelRow({ children: [Span({ children: "Training Mode" }), lblMode] }), selectMode, lblPendingSwitch] }),
          levelSlider.element,
          zoom.element,
          elite.element,
          mutation.element,
          inflow.element,
          hof.element,
          multiTrial.element,
          lamarck.element,
        ],
      }),
      ExporterCard({
        children: [
          H3({ children: [IconTrophy, "Champion Exporter (Local Cache)"] }),
          P({ children: "DNA of the best creature of the last generation. Seeding champions into the Ocean is planned (TCK-123)." }),
          txtDna,
          btnCopyDna,
        ],
      }),
    ],
  });

  const btnBack = BackToMenuButton({ children: [IconBack, "Main Menu"] });
  const grid = SandboxGridContainer({ id: "sandbox-grid" });
  const middleCol = MiddleContainer({
    children: [
      HeaderNavRow({ children: [H2({ style: { margin: "0", fontSize: "0.85rem", color: "#fff", textTransform: "uppercase" }, children: " Evolutionary Sandboxes" }), btnBack] }),
      grid,
    ],
  });

  const inspector = GlassPanel({ id: "trainer-inspect-panel", class: "panel-right" });
  root.appendChild(TrainerGrid({ children: [leftPanel, middleCol, inspector] }));

  return {
    root, btnBack, runTrigger, runCurrent, runList, txtNewRun, btnCreateRun,
    statGen: gen.el, statBestFit: best.el, statAvgFit: avg.el, statSuccess: success.el, statMedianFit: median.el,
    statFirstCatch: firstCatch.el, statLevel: level.el, statTimer: timer.el, canvasProgress, btnStart, btnReset,
    sliderPopulation: population.input, lblPopulation: population.lbl, sliderGridSize: gridSize.input, lblGridSize: gridSize.lbl,
    sliderSpeedup: speedup.input, lblSpeedup: speedup.lbl, selectMode, lblMode, lblPendingSwitch,
    sliderLevel: levelSlider.input, lblLevel: levelSlider.lbl, sliderZoomTiles: zoom.input, lblZoomTiles: zoom.lbl,
    sliderEliteRatio: elite.input, lblEliteRatio: elite.lbl, sliderMutation: mutation.input, lblMutation: mutation.lbl,
    sliderInflow: inflow.input, lblInflow: inflow.lbl, sliderHof: hof.input, lblHof: hof.lbl,
    chkMultiTrial: multiTrial.input, chkLamarck: lamarck.input, txtDna, btnCopyDna, grid, inspector,
  };
}
