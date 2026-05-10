pub struct SharedBanks([[u8; 256]; 2]);

impl std::ops::Deref for SharedBanks {
    type Target = [[u8; 256]; 2];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
