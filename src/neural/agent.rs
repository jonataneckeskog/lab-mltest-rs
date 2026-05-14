use crate::neural::{
    config::MutationSettings,
    genome::Genome,
    memory::{PrivateBanks, SharedBanks},
};
use crate::vm::traits::VmMemory;
use ordered_float::OrderedFloat;

#[derive(Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct Agent {
    pub(crate) base_genome: Genome,
    pub(crate) genome: Vec<u8>,
    pub(crate) private_banks: PrivateBanks,
    pub(crate) energy: OrderedFloat<f32>,
    pub age: u64,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            base_genome: Genome::default(),
            genome: Vec::with_capacity(32),
            private_banks: PrivateBanks::default(),
            energy: OrderedFloat(0.0),
            age: 0,
        }
    }
}

impl Agent {
    pub fn set_mutation_settings(&mut self, settings: MutationSettings) {
        let bank = self.private_banks.raw_mut(5);
        bank[0] = settings.cosmic_ray_rate;
    }

    pub fn get_mutation_settings(&self) -> MutationSettings {
        let bank = &self.private_banks.0[5];
        MutationSettings {
            cosmic_ray_rate: bank[0],
        }
    }

    pub fn load_input(&mut self, data: &[u8]) {
        self.private_banks.write_input(data);
    }

    pub fn collect_output(&self) -> Vec<u8> {
        self.private_banks.read_output()
    }

    pub fn get_energy(&self) -> f32 {
        self.energy.0
    }

    pub fn set_energy(&mut self, val: f32) {
        self.energy.0 = val;
    }

    pub fn set_genome(&mut self, genome: Vec<u8>) {
        self.genome = genome;
    }

    pub fn get_genome(&self) -> &[u8] {
        &self.genome
    }
}

pub struct AgentVmMemory<'a> {
    pub agent: &'a mut Agent,
    pub shared: &'a mut SharedBanks,
    bank_ptrs: [*mut u8; 8],
}

impl<'a> AgentVmMemory<'a> {
    pub fn new(agent: &'a mut Agent, shared: &'a mut SharedBanks) -> Self {
        let bank_ptrs = [
            agent.private_banks.raw_mut(0).as_mut_ptr(),
            agent.private_banks.raw_mut(1).as_mut_ptr(),
            agent.private_banks.raw_mut(2).as_mut_ptr(),
            agent.private_banks.raw_mut(3).as_mut_ptr(),
            agent.private_banks.raw_mut(4).as_mut_ptr(),
            agent.private_banks.raw_mut(5).as_mut_ptr(),
            shared.raw_mut(0).as_mut_ptr(),
            shared.raw_mut(1).as_mut_ptr(),
        ];
        Self {
            agent,
            shared,
            bank_ptrs,
        }
    }
}

impl<'a> VmMemory for AgentVmMemory<'a> {
    fn read_genome(&self, pc: usize) -> u8 {
        self.agent.genome.get(pc).copied().unwrap_or(0)
    }

    fn write_genome(&mut self, pc: usize, val: u8) {
        if let Some(byte) = self.agent.genome.get_mut(pc) {
            *byte = val;
        }
    }

    fn genome_len(&self) -> usize {
        self.agent.genome.len()
    }

    fn resize_genome(&mut self, new_size: usize) {
        self.agent.genome.resize(new_size, 0);
    }

    #[inline(always)]
    fn read_bank(&self, bank_idx: usize, addr: u8) -> u8 {
        unsafe { *self.bank_ptrs[bank_idx & 7].add(addr as usize) }
    }

    #[inline(always)]
    fn write_bank(&mut self, bank_idx: usize, addr: u8, val: u8) {
        unsafe { *self.bank_ptrs[bank_idx & 7].add(addr as usize) = val };
    }

    fn get_energy(&self) -> f32 {
        self.agent.energy.0
    }

    fn get_age(&self) -> u64 {
        self.agent.age
    }

    fn consume_energy(&mut self, amount: f32) {
        let metabolic_tax = (1.0 - (self.agent.age as f32 * 0.00000001)).max(0.0);
        self.agent.energy.0 = ((self.agent.energy.0 - amount) * metabolic_tax).max(0.0);
    }
}
