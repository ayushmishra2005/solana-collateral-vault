use solana_sdk::pubkey::Pubkey;
use crate::error::Result;

pub struct CPIManager {
    #[allow(dead_code)]
    program_id: Pubkey,
    authorized_programs: Vec<Pubkey>,
}

impl CPIManager {
    pub fn new(program_id: Pubkey, authorized_programs: Vec<Pubkey>) -> Self {
        Self {
            program_id,
            authorized_programs,
        }
    }

    pub async fn lock_collateral(
        &self,
        _user: &Pubkey,
        _amount: u64,
    ) -> Result<String> {
        Ok("transaction_signature".to_string())
    }

    pub async fn unlock_collateral(
        &self,
        _user: &Pubkey,
        _amount: u64,
    ) -> Result<String> {
        Ok("transaction_signature".to_string())
    }

    pub fn is_authorized(&self, program: &Pubkey) -> bool {
        self.authorized_programs.contains(program)
    }
}

