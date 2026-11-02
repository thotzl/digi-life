//! Brain system: one CTRNN step with live Hebbian learning (`shared::brain::execute_brain_with_learning`).

use crate::biology::brain::MotorOutputs;
use crate::shared::brain::execute_brain_with_learning;
use crate::shared::dto::CreatureAgent;
use crate::sim::TickCtx;

/// Runs the agent's brain on `inputs` with the config's Hebbian rate (`brain.hebbianLearningRateBase`; probes pass a
/// config with rate 0 to switch learning off) and writes the motor outputs.
pub fn think(agent: &mut CreatureAgent, inputs: &[f32], ctx: &TickCtx, motors: &mut MotorOutputs) {
    *motors = execute_brain_with_learning(
        &agent.phenotype.brain,
        inputs,
        &mut agent.neuron_states,
        &mut agent.neuron_activations,
        &mut agent.synapse_weights,
        &mut agent.brain_runtime,
        ctx.brain.hebbian_learning_rate_base,
    );
}
