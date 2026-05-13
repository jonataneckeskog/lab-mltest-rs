use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, PartialOrd, Ord)]
pub struct Bank<const N: usize>(#[serde_as(as = "[Bytes; N]")] pub(crate) [[u8; 256]; N]);

pub trait BankMetadata {
    const PREFIX: &'static str;
}

pub type PrivateBanks = Bank<6>;
pub type SharedBanks = Bank<2>;

impl BankMetadata for Bank<6> {
    const PREFIX: &'static str = "private_banks";
}
impl BankMetadata for Bank<2> {
    const PREFIX: &'static str = "shared_banks";
}

impl<const N: usize> Default for Bank<N> {
    fn default() -> Self {
        Bank([[0u8; 256]; N])
    }
}

impl<const N: usize> Bank<N> {
    pub fn raw_mut(&mut self, bank_idx: usize) -> &mut [u8; 256] {
        &mut self.0[bank_idx]
    }
}

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
}
