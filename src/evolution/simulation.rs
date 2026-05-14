use crate::sim::Multiverse;
use crate::sim::SingleStepTask;
use crate::sim::SimulationRunner;
use crate::neural::AgentSpawner;
use crate::vm::AgentExecutor;
use crate::neural::config::OP_COSTS;
use std::path::PathBuf;

pub struct EvolutionConfig {
    pub communities: usize,
    pub min_population: usize,
    pub starting_energy: f32,
    pub max_generations: usize,
    pub tick_energy_budget: f32,
    pub ticks_per_gen: usize,
    pub save_interval: Option<usize>,
    pub checkpoint_dir: Option<PathBuf>,
}

pub struct EvolutionEngine<'a> {
    pub config: EvolutionConfig,
    pub multiverse: Multiverse,
    pub task: &'a dyn SingleStepTask,
    pub executor: AgentExecutor<'a>,
    pub spawner: AgentSpawner,
}

pub trait EvolutionHook {
    fn on_generation_complete(&mut self, generation: usize, multiverse: &Multiverse) -> bool;
}

impl<T: EvolutionHook> EvolutionHook for &mut T {
    fn on_generation_complete(&mut self, generation: usize, multiverse: &Multiverse) -> bool {
        (**self).on_generation_complete(generation, multiverse)
    }
}

impl<'a> EvolutionEngine<'a> {
    pub fn new(
        config: EvolutionConfig,
        multiverse: Multiverse,
        task: &'a dyn SingleStepTask,
        spawner: AgentSpawner,
    ) -> Self {
        let executor = AgentExecutor::new(&OP_COSTS);
        Self {
            config,
            multiverse,
            task,
            executor,
            spawner,
        }
    }

    pub fn run<H: EvolutionHook>(&mut self, rng: &mut impl rand::Rng, mut hook: H) -> anyhow::Result<()> {
        let runner = SimulationRunner::new(&self.executor);

        for generation in 0..=self.config.max_generations {
            runner.run_population_tick(
                &mut self.multiverse,
                self.task,
                self.config.tick_energy_budget,
                self.config.ticks_per_gen,
                self.config.min_population,
                || self.spawner.new_random(rng),
            );

            runner.mutate(rng, &mut self.multiverse);

            let should_continue = hook.on_generation_complete(generation, &self.multiverse);

            if let Some(interval) = self.config.save_interval {
                if generation > 0 && generation % interval == 0 {
                    if let Some(ref dir) = self.config.checkpoint_dir {
                        let path = dir.join(format!("gen_{}", generation));
                        self.multiverse.save_to(path.to_str().unwrap())?;
                    }
                }
            }

            if self.multiverse.survivor_count() == 0 || !should_continue {
                break;
            }
        }

        Ok(())
    }
}
