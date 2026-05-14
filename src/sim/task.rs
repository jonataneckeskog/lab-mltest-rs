/// A simple one-shot task where agents receive input and are evaluated once.
pub trait SingleStepTask {
    /// Provide input data for the agents.
    fn input_data(&self) -> &[u8];
    
    /// Evaluate the agent's output.
    fn evaluate(&self, output: &[u8]) -> f32;
}

/// A stateful task that involves multiple steps of interaction.
pub trait MultiStepTask {
    /// Provide input for the current step.
    fn next_input(&mut self) -> &[u8];
    
    /// Evaluate output for the current step. Returns current score/reward.
    fn step(&mut self, output: &[u8]) -> f32;
    
    /// Whether the task is finished.
    fn is_finished(&self) -> bool;
}
