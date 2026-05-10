pub struct PrivateBanks([[u8; 256]; 6]);

impl std::ops::Deref for PrivateBanks {
    type Target = [[u8; 256]; 6];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
