use crate::core::SingleStepTask;

pub struct ConstantTask {
    pub target: u8,
}

impl SingleStepTask for ConstantTask {
    fn input_data(&self) -> &[u8] {
        &[]
    }
    fn evaluate(&self, output: &[u8]) -> f32 {
        if output.is_empty() {
            return 0.0;
        }
        let mut score = 0.1;
        let diff = (output[0] as i16 - self.target as i16).abs();
        score += (1.0 - (diff as f32 / 255.0)) * 0.4;
        if output[0] == self.target {
            score += 0.5;
        }
        score
    }
}
