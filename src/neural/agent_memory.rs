pub struct PrivateBanks([[u8; 256]; 6]);

impl PrivateBanks {
    pub fn write_input(&mut self, data: &[u8]) {
        let bank = &mut self.0[0];
        let len = data.len().min(255);
        bank[0] = len as u8;

        // Copy the valid range to agent memory
        bank[1..1 + len].copy_from_slice(&data[..len]);
    }

    pub fn read_output(&self) -> Vec<u8> {
        let bank = &self.0[1];
        let len = bank[0] as usize;
        let end = (1 + len).min(256);

        // Output the valid range of the output bank
        bank[1..end].to_vec()
    }

    // This allows the Agent's internal VM to still touch raw bytes
    pub fn raw_mut(&mut self, bank_idx: usize) -> &mut [u8; 256] {
        &mut self.0[bank_idx]
    }
}

pub struct SharedBanks([[u8; 256]; 2]);
