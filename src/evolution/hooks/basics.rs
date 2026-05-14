use std::path::PathBuf;
use crate::sim::Multiverse;
use crate::evolution::EvolutionHook;

pub struct CheckpointHook {
    pub interval: usize,
    pub dir: PathBuf,
}

impl EvolutionHook for CheckpointHook {
    fn on_generation_complete(&mut self, generation: usize, multiverse: &Multiverse) -> bool {
        if generation > 0 && generation % self.interval == 0 {
            let path = self.dir.join(format!("gen_{}.checkpoint", generation));
            let _ = multiverse.save_to(&path);
        }
        true
    }
}

pub struct PrintStatsHook {
    pub interval: usize,
    pub highest_survivors: usize,
}

impl EvolutionHook for PrintStatsHook {
    fn on_generation_complete(&mut self, generation: usize, multiverse: &Multiverse) -> bool {
        let survivor_count = multiverse.survivor_count();
        let (_min_e, max_e, avg_e) = multiverse.get_energy_stats();

        if generation % self.interval == 0 || survivor_count > self.highest_survivors {
            println!(
                "Gen {:03} | Survivors: {:4} | Max Energy: {:.2} | Avg: {:.2}",
                generation, survivor_count, max_e, avg_e
            );
        }

        if survivor_count > self.highest_survivors {
            self.highest_survivors = survivor_count;
            println!(">> 🏆 New survivor record!");
        }

        true
    }
}

pub struct PopulationTargetHook {
    pub target: usize,
}

impl EvolutionHook for PopulationTargetHook {
    fn on_generation_complete(&mut self, _generation: usize, multiverse: &Multiverse) -> bool {
        let survivor_count = multiverse.survivor_count();
        if survivor_count >= self.target {
            println!("🎯 Convergence reached ({} survivors)!", survivor_count);
            return false;
        }
        true
    }
}
