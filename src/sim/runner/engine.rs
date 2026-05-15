use crate::vm::AgentExecutor;

pub struct SimulationRunner<'a> {
    pub executor: &'a AgentExecutor<'a>,
}

impl<'a> SimulationRunner<'a> {
    pub fn new(executor: &'a AgentExecutor<'a>) -> Self {
        Self { executor }
    }
}
