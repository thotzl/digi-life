---
name: biology-dna
description: Decodes the biological mechanics, genome parsing, Continuous-Time Recurrent Neural Networks (CTRNN) brain, and phenotype properties of creatures in Pixel DNA Life.
---
# biology-dna: Biology, Genetics & CTRNN Brain Simulation

## I. Genomic Foundation
- **Sense String (DNA):** Standard uppercase alphabetical characters `A-Z`.
- **Antisense String:** Complementary base pairing of DNA strands to simulate true biological redundancy.
- **Phenotypic De-compilation:** The genome encodes HSL colors, spinal geometry, sensory organs (photoreceptors, chemoreceptors, mechanoreceptors), behavioral predispositions (e.g., carnivory Index), and r/K reproduction strategies.

## II. Neural Architecture (CTRNN Brain)
- **Model:** Continuous-Time Recurrent Neural Network (CTRNN) mapped directly from the genome topology.
- **Nodes & Synapses:**
  - **Inputs:** Environmental sensors (food direction, prey proximity, light gradients, internal energy/adrenaline).
  - **Outputs:** Physical effectors (propulsion speed, rotatory steering angle, mitotic splitting, attacking, and feeding).
  - **Hidden layers:** Recurrent structures permitting memory and oscillation.
- **Mathematical Integration:** Solved via Euler numerical integration on the server during each frame step:
  $$\tau_i \frac{dy_i}{dt} = -y_i + \sum w_{ji} \sigma(y_j + \theta_j) + I_i$$
  Where:
  - $\tau_i$ is the node's time constant (decay).
  - $\theta_j$ is the bias of the presynaptic node.
  - $\sigma(x)$ is the standard sigmoid activation function.

## III. Evolutionary Dynamics
- **Speciation:** Logged in the virtual `SpeciesRecord` database (`species_db.json`). When species drift genetically beyond a threshold, they are bifurcated into a new species with a registered Latin binomial name (lineage tracker).
- **Reproduction Heuristics:** Organisms reproduce via Mitosis (splitting) if they cross their age of maturation (`matureAge`), accumulate sufficient stomach energy threshold (`reproThreshold`), and satisfy the mandatory physiological feeding guarantee (`hasEaten === true`).
