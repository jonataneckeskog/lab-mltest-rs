// Formula for Balanced Complexity:
// TotalEnergy = (TargetCPU / CurrentCPU)^p * log2(N + 1) * K
//
// - (TargetCPU / CurrentCPU)^p: The Governor (punishes big/slow agents via actual CPU load)
// - log2(N + 1): The Diversity Reward (rewards having "enough" agents, then tapers off)
// - K: Global scalar constant
pub fn compute_global_energy(
    current_cpu: f32,
    target_cpu: f32,
    total_agents: usize,
    k: f32,
    p: f32,
) -> f32 {
    let n = total_agents as f32;

    if n == 0.0 {
        return k;
    }

    // Governor: Punishes based on actual CPU load (where 'Big' agents naturally hit harder)
    let governor = (target_cpu / current_cpu.max(0.001)).powf(p);

    // Population Reward: log2 provides a "Satiation" curve.
    // 1 -> 80 agents is a massive bonus; 80 -> 5000 is a diminishing return.
    let population_reward = (n + 1.0).log2();

    governor * population_reward * k
}
