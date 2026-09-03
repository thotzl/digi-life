import { styled } from "../core/styled";
import { navigateTo } from "../core/router";
import { 
  IconDashboard, IconLog, IconBack, IconPlus, IconReset 
} from "../components/Icons";
import { UnifiedDiagnosticsPanel } from "./UnifiedDiagnosticsPanel";

// --- STYLED COMPONENTS ---

const OceanContainer = styled("div")`
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: #020617;
`;

const BackgroundCanvas = styled("canvas")`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
`;

const DashboardGrid = styled("main")`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: grid;
  grid-template-columns: 320px 1fr 360px;
  grid-template-rows: 1fr 180px;
  grid-gap: 15px;
  padding: 15px;
  box-sizing: border-box;
  z-index: 10;
  pointer-events: none; /* Let clicks pass through grid spaces to canvas */

  @media (max-width: 1200px) {
    grid-template-columns: 280px 1fr 300px;
    grid-template-rows: 1fr 140px;
  }
`;

const LeftGlassPanel = styled("section")`
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
  pointer-events: auto;
  grid-column: 1;
  grid-row: 1 / 3; /* Full height across both rows! */
`;

const RightGlassPanel = styled("section")`
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
  pointer-events: auto;
  grid-column: 3;
  grid-row: 1 / 3; /* Full height across both rows! */
`;

const BottomGlassPanel = styled("section")`
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
  pointer-events: auto;
  grid-column: 2;
  grid-row: 2; /* Positioned exactly in the middle at the bottom! */
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
  letter-spacing: 1px;
  margin-top: 2px;
  display: block;
`;

const StatsCard = styled("div")`
  background: rgba(15, 23, 42, 0.45);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
`;

const StatsItem = styled("div")`
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.7rem;
  font-family: monospace;
`;

const StatsLabel = styled("span")`
  color: #94a3b8;
`;

const StatsValue = styled("span")`
  font-weight: bold;
  &.val-green { color: #10b981; text-shadow: 0 0 6px rgba(16, 185, 129, 0.35); }
  &.val-blue { color: #00f2fe; text-shadow: 0 0 6px rgba(0, 242, 254, 0.35); }
  &.val-gold { color: #f59e0b; text-shadow: 0 0 6px rgba(245, 158, 11, 0.35); }
`;

const ButtonGroup = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 15px;
`;

const CyberButton = styled("button")`
  background: rgba(0, 242, 254, 0.05);
  border: 1px solid rgba(0, 242, 254, 0.2);
  color: #00f2fe;
  padding: 8px 12px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.7rem;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
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

const SliderControl = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
  margin-bottom: 4px;
`;

const SliderLabelRow = styled("label")`
  display: flex;
  justify-content: space-between;
  font-size: 0.65rem;
  color: #94a3b8;
  font-family: monospace;
`;

const SliderInput = styled("input")`
  width: 100%;
  background: rgba(148, 163, 184, 0.1);
  height: 6px;
  border-radius: 3px;
  outline: none;
  cursor: pointer;
  margin-bottom: 4px;
`;

const PanelSection = styled("div")`
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
`;

const SectionTitle = styled("h3")`
  font-size: 0.72rem;
  color: #94a3b8;
  margin: 0 0 8px 0;
  text-transform: uppercase;
  letter-spacing: 1px;
`;

const SpeciesListContainer = styled("div")`
  flex: 1;
  overflow-y: auto;
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  background: rgba(15, 23, 42, 0.3);
  padding: 6px;
`;

const TerminalFooterRow = styled("div")`
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  padding-bottom: 6px;
  margin-bottom: 8px;
`;

const ClearButton = styled("button")`
  background: none;
  border: none;
  outline: none;
  color: #64748b;
  font-family: monospace;
  font-size: 0.58rem;
  cursor: pointer;
  text-transform: uppercase;

  &:hover {
    color: #ef4444;
  }
`;

const LogStreamContainer = styled("div")`
  flex: 1;
  overflow-y: auto;
  font-family: monospace;
  font-size: 0.62rem;
  line-height: 1.4;
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const BackToMenuButton = styled("button")`
  position: absolute;
  top: 15px;
  left: 350px; /* Positioned nicely between sidebars */
  background: rgba(15, 23, 42, 0.7);
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 4px;
  color: #94a3b8;
  padding: 6px 12px;
  font-family: monospace;
  font-size: 0.65rem;
  cursor: pointer;
  z-index: 100;
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

export function createOceanView(): HTMLElement {
  const container = OceanContainer({ id: "ocean-view-container" });

  // 1. Interactive background canvas
  const canvas = BackgroundCanvas({ id: "creature-canvas" });
  container.appendChild(canvas);

  // 2. Navigation "Leave" Button
  const btnLeave = BackToMenuButton({
    onClick: () => navigateTo("main-menu"),
    children: [
      IconBack,
      "Main Menu"
    ]
  });
  container.appendChild(btnLeave);

  // 3. Assemble Dashboard Grid Panels
  const panelLeft = LeftGlassPanel({
    children: [
      PanelHeader({
        children: [
          PanelTitle({ 
            children: [
              IconDashboard,
              "Control Deck"
            ] 
          }),
          PanelSubtitle({ children: "Global Biosphere Telemetry" })
        ]
      }),
      StatsCard({
        children: [
          StatsItem({
            children: [
              StatsLabel({ children: "Organisms:" }),
              StatsValue({ id: "stat-population", class: "val-green", children: "0 / 25" })
            ]
          }),
          StatsItem({
            children: [
              StatsLabel({ children: "Highest Gen:" }),
              StatsValue({ id: "stat-generation", class: "val-blue", children: "1" })
            ]
          }),
          StatsItem({
            children: [
              StatsLabel({ children: "Nutrient Spores:" }),
              StatsValue({ id: "stat-spores", class: "val-gold", children: "0 Spores" })
            ]
          })
        ]
      }),
      ButtonGroup({
        children: [
          CyberButton({
            id: "btn-inject",
            children: [
              IconPlus,
              "Release Protocell"
            ]
          }),
          CyberButton({
            id: "btn-load-catalogue",
            style: { borderColor: "rgba(245, 158, 11, 0.3)", color: "#f59e0b", background: "rgba(245, 158, 11, 0.04)" },
            children: [
              IconPlus,
              "Load from Catalog"
            ]
          }),
          CyberButton({
            id: "btn-reset",
            class: "btn-danger",
            children: [
              IconReset,
              "Reset Evolution"
            ]
          }),
          CyberButton({
            id: "btn-toggle-sim",
            children: "Pause Substrate"
          }),
          SliderControl({
            children: [
              SliderLabelRow({
                children: [
                  styled("span")``({ children: "Warp Speed" }),
                  styled("span")``({ id: "lbl-ocean-speedup", children: "1x" })
                ]
              }),
              SliderInput({ id: "slider-ocean-speedup", type: "range", min: "1", max: "12", value: "1", step: "1" })
            ]
          })
        ]
      }),
      PanelSection({
        children: [
          SectionTitle({ children: "Registered Species" }),
          SpeciesListContainer({ id: "species-roster" })
        ]
      })
    ]
  });

  // Reusable, reactive, and completely encapsulated Diagnostics Panel!
  const panelRight = RightGlassPanel({
    id: "inspect-panel"
  });
  const diagnosticsPanel = new UnifiedDiagnosticsPanel("ocean");
  panelRight.appendChild(diagnosticsPanel.element);

  const panelBottom = BottomGlassPanel({
    children: [
      TerminalFooterRow({
        children: [
          PanelTitle({ 
            children: [
              IconLog,
              "Log Terminal"
            ] 
          }),
          ClearButton({ id: "btn-clear-console", children: "Clear Terminal" })
        ]
      }),
      LogStreamContainer({ id: "terminal-logs" })
    ]
  });

  const grid = DashboardGrid({
    children: [
      panelLeft,
      panelRight,
      panelBottom
    ]
  });

  container.appendChild(grid);

  return container;
}
