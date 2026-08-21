# Pixel DNA Life: Technical & Conceptual Whitepaper
## An Evolutionary Artificial Life & Competitive Multi-Agent Sandbox

---

## 1. Executive Summary & Vision

**Pixel DNA Life** is an upcoming artificial life simulation and decentralized competitive strategy game. It merges the rich, emergent biological realism of single-agent evolution simulators with the strategic, large-scale programming competition of persistent multiplayer worlds. 

In traditional programming games, players command their entities using explicit scripting languages (e.g., JavaScript). In **Pixel DNA Life**, players do not write code. Instead, they act as evolutionary engineers: designing, mutating, training, and breeding autonomous digital organisms. These organisms are controlled by a continuous-time neural network mapped from a double-stranded genetic sequence (DNA).

The core vision of Pixel DNA Life is divided into two distinct, interconnected lifecycles:
1. **The Local Laboratory:** A sandbox environment where players design, optimize, and train their biological lineages through selective breeding, manual gene-editing, and targeted fitness scenarios.
2. **The Global Ecosystem:** A distributed multiplayer environment where players deploy their highly-trained lineages onto remote, persistent servers. Here, their organisms must feed, adapt, reproduce, and strategically out-compete the creations of other players to achieve genetic and territorial dominance.

The gameplay draws philosophical inspiration from the organic evolution of *The Bibites* and the competitive, decentralized server colonization of *Screeps*, creating a brand new genre of evolutionary-driven strategy.

---

## 2. Genomic & Neural Architecture (The DNA Principle)

Every creature in Pixel DNA Life is entirely defined by its genetic code and controlled by a simulated brain. There is no artificial hardcoding of behaviors; actions are the pure, mathematical output of environmental stimulation passing through neural synapses.

### 2.1 Double-Stranded Genetics & Speciation
Each organism contains a **double-stranded genome** consisting of:
- **Sense String (DNA):** A sequence of standard uppercase alphabetical characters `A-Z`.
- **Antisense String:** A complementary base sequence reflecting true biological redundancy and assisting in complex mutation patterns.

The genome undergoes **phenotypic de-compilation** to dynamically determine the creature's physiology:
- **HSL Coloration:** Dynamic aesthetic profiles reflecting genetic origin.
- **Spinal Geometry:** Structural length, segment count, and physical proportions.
- **Sensory Layout:** Placement and sensitivity of photoreceptors, chemoreceptors, and mechanoreceptors.
- **Ecological Strategy:** Behavioral predispositions (e.g., Carnivory vs. Herbivory Index) and reproductive tendencies (r/K selection parameters).

As populations evolve, genetic distances are monitored. When a lineage drifts past a defined genetic threshold, the system triggers a **speciation event**, registering a new species with a distinct Latin binomial name in the shared database (`SpeciesRecord`) to track ancestral history and evolutionary lineages.

### 2.2 The CTRNN Brain
The creature's brain is modeled as a **Continuous-Time Recurrent Neural Network (CTRNN)**, mapped directly from its genome topology. 
- **Sensory Inputs:** Environmental values (food gradient vectors, nearby prey/predator proximity, light gradients, internal energy reserves, and adrenaline levels).
- **Physical Effectors (Outputs):** Direct actions (propulsion thrust, rotational steering angle, mitotic splitting trigger, attacking force, and feeding rate).
- **Recurrent Hidden Layers:** Self-referential neural connections that allow the network to maintain memory, build temporal patterns, and develop internal oscillations (e.g., swimming rhythms).

The physical and neural state is updated on the simulation server in real-time. The neuron activation states are integrated numerically using the Euler method for every simulation frame:

$$\tau_i \frac{dy_i}{dt} = -y_i + \sum w_{ji} \sigma(y_j + \theta_j) + I_i$$

Where:
- $\tau_i$ is the node's decay time constant.
- $w_{ji}$ represents synaptic weights.
- $\theta_j$ is the bias of the presynaptic node.
- $\sigma(x)$ is the standard sigmoid activation function.
- $I_i$ is the external sensory current injected into node $i$.

---

## 3. The Local Laboratory: Training & Evolution

Before launching creatures into the global wild, players use the offline **Local Laboratory** to cultivate their lineages. This environment is designed for rapid iteration, testing, and manual engineering.

### 3.1 The Creator
The **Creator** is a specialized laboratory workstation. It allows players to:
- **Generate Random Genomes:** Instantly spawn diverse, randomized gene-pools to kickstart custom evolution.
- **Simulate Targeted Mutations:** Apply artificial radiation or genetic drift to observe biological variance.
- **The DNA Editor:** A low-level sequence editor allowing players to manually write, copy, paste, and modify individual characters on the sense and antisense strands, explicitly crafting neural connections and physical traits.

### 3.2 Environmental Training Chambers (Rooms)
The local sandbox offers specialized training rooms, each simulating distinct physical constraints and ecological rules:
- **The Training Room:** A controlled environment with guided scenarios (e.g., navigation obstacles, static food sourcing) designed for foundational training.
- **The Ocean:** A fluid-dynamics optimized water chamber. It emphasizes efficient locomotion, momentum management, and complex swimming/hunting behaviors.
- **The Wilderness:** A resource-scarce, rugged land sandbox. Extreme conditions force creatures to optimize their r/K-selection strategies, minimize metabolic waste, and adapt to unpredictable food shortages.

### 3.3 The Local Training Arena
A sandbox module dedicated to competitive validation. Players can load different locally bred lineages and pit them against each other in custom competitive formats:
- **1v1 Duels:** Direct tactical confrontations between two species.
- **Deathmatch:** Multi-species free-for-all tests of general combat and survival capabilities.
- **Team-based Skirmishes:** Cohorts of organisms working (or co-evolving) to control resources.

---

## 4. The Global Multi-Agent Ecosystem

The ultimate benchmark of an evolutionary engineer is the deployment of their lineage to the persistent online ecosystem. This phase transitions the game from a single-player sandbox into a massive, asynchronous, multi-agent competition.

```
       [ Local Laboratory ]
     ┌──────────────────────┐
     │  - DNA / Gene Editor │
     │  - Training Chambers │
     │  - Local Arenas      │
     └──────────┬───────────┘
                │
         (Export Genome)
                │
                ▼
   [ Global Server Infrastructure ]
   ┌──────────────────────────────┐
   │                              │
   │  ┌────────────────────────┐  │
   │  │   Competitive Arenas   │  │
   │  │ (1v1, Deathmatch, etc) │  │
   │  └────────────────────────┘  │
   │                              │
   │  ┌────────────────────────┐  │
   │  │     The Persistent     │  │
   │  │         WORLD          │  │
   │  │  - Resource Domination │  │
   │  │  - Real-time Mitosis   │  │
   │  │  - Species Takeover    │  │
   │  └────────────────────────┘  │
   └──────────────────────────────┘
```

### 4.1 Competitive Arenas
A platform for matched competitive play. Players queue their genomes in standardized server environments. 
- **Deterministic Matchmaking:** Servers load player genomes and simulate the match in deterministic, high-speed headless cycles.
- **Formats:** Standardized 1v1, Team Deathmatches, or asymmetric survival scenarios.
- **Global Leaderboards:** Lineages earn Elo ratings and tactical badges based on their performance in these arenas.

### 4.2 The Persistent "World"
The pinnacle of Pixel DNA Life. **The World** is a massive, persistent, multiplayer server map where hundreds of players deploy their creatures.
- **Autonomous Colonization:** Once deployed, the creatures act fully autonomously. The player has no direct control over them. The deployed species must forage, hunt, avoid hazards, and reproduce.
- **Ecological Reproduction (Mitosis):** Deployed creatures divide via mitosis when they cross their maturation age, accumulate enough stomach energy, and satisfy a basic feeding history.
- **Decentralized Conquest:** Deployed genomes will mutate over generations. If a player’s initial design is structurally robust, their species will successfully secure food sources, establish territorial control, defend against rival species, and slowly take over whole server regions.
- **Genetic Conquest:** The server represents a true evolutionary battlefield where only the most adaptable CTRNN brains and genetic layouts survive. Players watch in real-time as their species either colonizes the global world or collapses into extinction.

---

## 5. System Architecture & High-Performance Headless Core

Pixel DNA Life utilizes a modern, decoupled web architecture designed to support heavy physical simulations and real-time multiplayer streaming without client bottlenecking.

### 5.1 Headless Server Engine
To ensure that complex physics and neural calculations are not throttled by browser rendering or tab-suspension, the entire simulation engine is designed as a **headless Node.js backend**.
- **Physics Core:** Calculated in ungedrosselt (warp-speed) cycles using an optimized Spatial Grid partition for instant collision detection.
- **Decoupled Main Loop:** The server performs fast physical ticks independent of the visual framerate, persisting structural logs and fitness histories directly into a high-performance database.

### 5.2 Real-time WebSocket Replication & Canvas Client
- **Sub-millisecond WebSocket Protocol (`ws`):** The server replicates positions, physical coordinates, and neural potential states to connected client frontends.
- **Passive Canvas Render Engine:** The web frontend (Vite, TypeScript, HTML5 Canvas, Preact Signals) acts as a passive consumer. It receives the high-frequency state replication from the server and renders it smoothly at high framerates.
- **Live Diagnostics HUD:** Dynamic UI overlays (Zustand, Preact Signals) allow real-time hover-diagnostics and neural activation visualization, drawing custom brain diagrams directly from the live websocket data stream.
