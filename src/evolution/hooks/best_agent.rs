use crate::evolution::EvolutionHook;
use crate::sim::Multiverse;

pub struct BestAgentHook {
    pub highest_survivors: usize,
    pub path: String,
}

impl EvolutionHook for BestAgentHook {
    fn on_generation_complete(&mut self, _generation: usize, multiverse: &Multiverse) -> bool {
        let survivor_count = multiverse.population;
        if survivor_count > self.highest_survivors {
            self.highest_survivors = survivor_count;
            let _ = multiverse.save_to(&self.path);
        }
        true
    }
}
