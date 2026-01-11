use solana_sdk::pubkey::Pubkey;
use solana_sdk::instruction::Instruction;

const INITIALIZE_VAULT_DISCRIMINATOR: [u8; 8] = [48, 191, 163, 44, 71, 129, 63, 164];
const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
const WITHDRAW_DISCRIMINATOR: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];

pub struct TransactionBuilder {
    program_id: Pubkey,
}

impl TransactionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id }
    }

    pub fn build_initialize_vault_instruction(
        &self,
        user: Pubkey,
        vault: Pubkey,
        vault_token_account: Pubkey,
        mint: Pubkey,
        vault_authority_pda: Pubkey,
        vault_authority: Pubkey,
    ) -> Instruction {
        let data = INITIALIZE_VAULT_DISCRIMINATOR.to_vec();
        // No additional args for initialize_vault

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(user, true),
                solana_sdk::instruction::AccountMeta::new(vault, false),
                solana_sdk::instruction::AccountMeta::new(vault_token_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(mint, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault_authority_pda, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault_authority, false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_token::ID, false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_associated_token_account::ID, false),
                solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data,
        }
    }

    pub fn build_deposit_instruction(
        &self,
        user: Pubkey,
        vault: Pubkey,
        user_token_account: Pubkey,
        vault_token_account: Pubkey,
        mint: Pubkey,
        vault_authority: Pubkey,
        amount: u64,
    ) -> Instruction {
        let mut data = DEPOSIT_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(user, true),
                solana_sdk::instruction::AccountMeta::new(vault, false),
                solana_sdk::instruction::AccountMeta::new(user_token_account, false),
                solana_sdk::instruction::AccountMeta::new(vault_token_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(mint, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault_authority, false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data,
        }
    }

    pub fn build_withdraw_instruction(
        &self,
        user: Pubkey,
        vault: Pubkey,
        user_token_account: Pubkey,
        vault_token_account: Pubkey,
        mint: Pubkey,
        vault_authority_pda: Pubkey,
        vault_authority: Pubkey,
        amount: u64,
    ) -> Instruction {
        let mut data = WITHDRAW_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(user, true),
                solana_sdk::instruction::AccountMeta::new(vault, false),
                solana_sdk::instruction::AccountMeta::new(user_token_account, false),
                solana_sdk::instruction::AccountMeta::new(vault_token_account, false),
                solana_sdk::instruction::AccountMeta::new_readonly(mint, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault_authority_pda, false),
                solana_sdk::instruction::AccountMeta::new_readonly(vault_authority, false),
                solana_sdk::instruction::AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data,
        }
    }
}
