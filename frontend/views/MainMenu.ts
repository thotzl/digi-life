import { styled } from "../core/styled";
import { navigateTo } from "../core/router";
import { IconDna, IconWeights, IconCatalogue } from "../components/Icons";

// --- STYLED COMPONENTS ---

const MenuContainer = styled("div")`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  width: 100vw;
  background: radial-gradient(circle at center, #0f172a 0%, #020617 100%);
  color: #f1f5f9;
  font-family: monospace;
  box-sizing: border-box;
  padding: 40px;
  position: relative;
  overflow: hidden;

  &::before {
    content: "";
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    background-image: linear-gradient(rgba(0, 242, 254, 0.015) 1px, transparent 1px),
                      linear-gradient(90deg, rgba(0, 242, 254, 0.015) 1px, transparent 1px);
    background-size: 40px 40px;
    background-position: center;
    pointer-events: none;
  }
`;

const HeaderBlock = styled("div")`
  text-align: center;
  margin-bottom: 60px;
  z-index: 10;
`;

const Title = styled("h1")`
  font-size: 3.2rem;
  font-weight: 900;
  letter-spacing: 8px;
  margin: 0 0 12px 0;
  background: linear-gradient(135deg, #00f2fe 0%, #4facfe 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 20px rgba(0, 242, 254, 0.4));
  text-transform: uppercase;
`;

const Subtitle = styled("p")`
  font-size: 0.72rem;
  color: #64748b;
  letter-spacing: 3px;
  margin: 0;
  text-transform: uppercase;
`;

const LaunchersGrid = styled("div")`
  display: grid;
  grid-template-columns: repeat(3, 320px);
  grid-gap: 24px;
  z-index: 10;
  margin-bottom: 60px;
  justify-content: center;

  @media (max-width: 1050px) {
    grid-template-columns: 1fr;
    grid-gap: 16px;
  }
`;

const LauncherCard = styled("button")`
  background: rgba(15, 23, 42, 0.55);
  border: 1px solid rgba(148, 163, 184, 0.12);
  border-radius: 8px;
  padding: 24px 20px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  text-align: left;
  cursor: pointer;
  outline: none;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
  height: 200px;
  box-sizing: border-box;

  /* Custom Hover Glow Themes for each module */
  &.card-ocean:hover {
    border-color: #10b981;
    background: rgba(16, 185, 129, 0.04);
    box-shadow: 0 0 25px rgba(16, 185, 129, 0.12);
    transform: translateY(-4px);

    svg {
      color: #10b981;
      filter: drop-shadow(0 0 8px rgba(16, 185, 129, 0.5));
    }
  }

  &.card-trainer:hover {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.04);
    box-shadow: 0 0 25px rgba(59, 130, 246, 0.12);
    transform: translateY(-4px);

    svg {
      color: #3b82f6;
      filter: drop-shadow(0 0 8px rgba(59, 130, 246, 0.5));
    }
  }

  &.card-catalogue:hover {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.04);
    box-shadow: 0 0 25px rgba(245, 158, 11, 0.12);
    transform: translateY(-4px);

    svg {
      color: #f59e0b;
      filter: drop-shadow(0 0 8px rgba(245, 158, 11, 0.5));
    }
  }

  &:active {
    transform: translateY(1px);
  }
`;

const CardIcon = styled("div")`
  margin-bottom: 18px;
  color: #475569;
  transition: all 0.25s ease;
`;

const CardTitle = styled("h2")`
  font-size: 1.05rem;
  font-weight: bold;
  margin: 0 0 10px 0;
  color: #f1f5f9;
  letter-spacing: 0.5px;
  text-transform: uppercase;
`;

const CardDesc = styled("p")`
  font-size: 0.68rem;
  color: #64748b;
  margin: 0;
  line-height: 1.45;
`;

const FooterBlock = styled("footer")`
  font-size: 0.62rem;
  color: #334155;
  letter-spacing: 1.5px;
  z-index: 10;
`;

// --- COMPONENT CREATOR ---

export function createMainMenu(): HTMLElement {
  const container = MenuContainer();

  const header = HeaderBlock({
    children: [
      Title({ children: "Pixel DNA" }),
      Subtitle({ children: "Evolutionary Cybernetic Substrate" })
    ]
  });

  const btnOcean = LauncherCard({
    class: "card-ocean",
    onClick: () => navigateTo("ocean"),
    children: [
      CardIcon({ children: IconDna }),
      CardTitle({ children: "Ocean Laboratory" }),
      CardDesc({ children: "Observe and control a continuously mutating biological reef ecosystem." })
    ]
  });

  const btnTrainer = LauncherCard({
    class: "card-trainer",
    onClick: () => navigateTo("trainer"),
    children: [
      CardIcon({ children: IconWeights }),
      CardTitle({ children: "Training Room" }),
      CardDesc({ children: "Breed and optimize specialized neural survivalists in parallel sandboxes." })
    ]
  });

  const btnCatalogue = LauncherCard({
    class: "card-catalogue",
    onClick: () => navigateTo("catalogue"),
    children: [
      CardIcon({ children: IconCatalogue }),
      CardTitle({ children: "Creature Catalog" }),
      CardDesc({ children: "Manage your persistent cryo-clones and load them with or without a learned brain." })
    ]
  });

  // Render launchers grid
  const grid = LaunchersGrid({
    children: [
      btnOcean,
      btnTrainer,
      btnCatalogue
    ]
  });

  const footer = FooterBlock({
    children: "TAURI DESKTOP ENGINE v2.0 // DECENTRALIZED GENETIC LABORATORY"
  });

  // Assemble Main Menu SPA View
  container.appendChild(header);
  container.appendChild(grid);
  container.appendChild(footer);

  return container;
}
