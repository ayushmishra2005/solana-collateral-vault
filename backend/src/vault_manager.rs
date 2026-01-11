use solana_sdk::pubkey::Pubkey;
use anchor_client::Client;
use crate::models::VaultInfo;
use crate::error::{Error, Result};
use crate::database::Database;
use crate::transaction_builder::TransactionBuilder;
use std::str::FromStr;
use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

macro_rules! require {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error);
        }
    };
}

pub struct VaultManager {
    rpc_client: Arc<RpcClient>,
    program_id: Pubkey,
    database: Database,
    mint: Pubkey,
    payer: Arc<Keypair>, // Admin wallet for signing transactions
    tx_builder: TransactionBuilder,
}

impl VaultManager {
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn new<C>(
        _client: Arc<Client<C>>,
        program_id: Pubkey,
        database: Database,
        mint: Pubkey,
    ) -> Self {
        let rpc_url = std::env::var("RPC_URL")
            .unwrap_or_else(|_| "http://localhost:8899".to_string());
        
        let rpc_client = Arc::new(RpcClient::new(rpc_url.clone()));
        
        let payer = {
            let keypair_path = if let Ok(path) = std::env::var("WALLET_PATH") {
                path
            } else {
                format!("{}/.config/solana/id.json", std::env::var("HOME").unwrap_or_default())
            };

            match std::fs::read_to_string(&keypair_path) {
                Ok(contents) => {
                    let keypair_bytes: Vec<u8> = serde_json::from_str(&contents)
                        .unwrap_or_else(|_| std::fs::read(&keypair_path).unwrap_or_default());
                    
                    if keypair_bytes.len() == 64 {
                        Arc::new(Keypair::from_bytes(&keypair_bytes).unwrap_or_else(|_| Keypair::new()))
                    } else {
                        Arc::new(Keypair::new())
                    }
                }
                Err(_) => Arc::new(Keypair::new())
            }
        };
        
        Self {
            rpc_client: rpc_client.clone(),
            program_id,
            database,
            mint,
            payer,
            tx_builder: TransactionBuilder::new(program_id),
        }
    }

    pub async fn initialize_vault(&self, user: &str) -> Result<String> {
        let user_pubkey = Pubkey::from_str(user)
            .map_err(|e| Error::InvalidAccount(format!("Invalid user pubkey: {}", e)))?;

        if user_pubkey != self.payer.pubkey() {
            return Err(Error::TransactionFailed(
                format!("For local testing, user pubkey must match admin wallet (the wallet used to run the server). Your admin wallet pubkey is: {}. In production, users would sign their own transactions.", self.payer.pubkey())
            ));
        }

        let rpc_client_check = self.rpc_client.clone();
        let payer_pubkey = self.payer.pubkey();
        let balance = tokio::task::spawn_blocking(move || {
            rpc_client_check.get_balance(&payer_pubkey)
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
        .map_err(|e| Error::SolanaClient(format!("Failed to get balance: {}", e)))?;

        if balance < 1_000_000 {
            let rpc_client_airdrop = self.rpc_client.clone();
            let payer_pubkey_airdrop = self.payer.pubkey();
            let airdrop_amount = 2_000_000_000;
            let airdrop_sig = tokio::task::spawn_blocking(move || {
                rpc_client_airdrop.request_airdrop(&payer_pubkey_airdrop, airdrop_amount)
            })
            .await
            .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
            .map_err(|e| Error::SolanaClient(format!("Failed to request airdrop: {}", e)))?;
            
            let rpc_client_confirm = self.rpc_client.clone();
            tokio::task::spawn_blocking(move || {
                for _ in 0..30 {
                    if let Ok(true) = rpc_client_confirm.confirm_transaction(&airdrop_sig) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            })
            .await
            .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?;
        }

        let rpc_client_mint = self.rpc_client.clone();
        let mint_pubkey = self.mint;
        let mint_exists = tokio::task::spawn_blocking(move || {
            rpc_client_mint.get_account(&mint_pubkey)
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?;

        if mint_exists.is_err() {
            return Err(Error::TransactionFailed(
                format!("Mint account {} does not exist on the local validator. Please create a test mint first or use a different mint address. For local testing, you can create a mint using: spl-token create-token --decimals 6", self.mint)
            ));
        }

        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let rpc_client = self.rpc_client.clone();
        let vault_pda_clone = vault_pda;
        match tokio::task::spawn_blocking(move || {
            rpc_client.get_account(&vault_pda_clone)
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))? {
            Ok(account) => {
                if !account.data.is_empty() {
                    return Err(Error::InvalidAccount("Vault already exists".to_string()));
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("AccountNotFound") || error_msg.contains("account not found") {
                } else {
                }
            }
        }

        let (vault_authority_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let (global_vault_authority, _) = Pubkey::find_program_address(
            &[b"vault_authority"],
            &self.program_id,
        );

        let vault_token_account = get_associated_token_address(&vault_authority_pda, &self.mint);

        let instruction = self.tx_builder.build_initialize_vault_instruction(
            user_pubkey,
            vault_pda,
            vault_token_account,
            self.mint,
            vault_authority_pda,
            global_vault_authority,
        );

        let recent_blockhash = tokio::task::spawn_blocking({
            let rpc_client = self.rpc_client.clone();
            move || rpc_client.get_latest_blockhash()
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
        .map_err(|e| Error::SolanaClient(format!("Failed to get blockhash: {}", e)))?;

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&user_pubkey),
        );
        transaction.message.recent_blockhash = recent_blockhash;

        if user_pubkey == self.payer.pubkey() {
            transaction.try_sign(&[&*self.payer], recent_blockhash)
                .map_err(|e| Error::TransactionFailed(format!("Failed to sign transaction: {}", e)))?;
        } else {
            return Err(Error::TransactionFailed(
                format!("Initialize requires user's wallet signature. User pubkey {} does not match admin wallet {}. Please use the admin wallet or provide your wallet for signing.", user_pubkey, self.payer.pubkey())
            ));
        }

        let rpc_client = self.rpc_client.clone();
        let signed_transaction = transaction.clone();
        let signature = tokio::task::spawn_blocking(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                rpc_client.send_and_confirm_transaction(&signed_transaction)
            })).unwrap_or_else(|_| {
                Err(solana_client::client_error::ClientError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Transaction send panicked"
                )))
            })
        })
        .await
        .map_err(|e| Error::TransactionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| Error::TransactionFailed(format!("Failed to send transaction: {}", e)))?;


        Ok(signature.to_string())
    }

    pub async fn deposit(&self, user: &str, amount: u64) -> Result<String> {
        let user_pubkey = Pubkey::from_str(user)
            .map_err(|e| Error::InvalidAccount(format!("Invalid user pubkey: {}", e)))?;

        require!(amount > 0, Error::InvalidAccount("Amount must be greater than 0".to_string()));

        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let rpc_client = self.rpc_client.clone();
        let vault_pda_clone = vault_pda;
        let account_info = tokio::task::spawn_blocking(move || {
            rpc_client.get_account(&vault_pda_clone)
        })
        .await
        .map_err(|_| Error::VaultNotFound)?
        .map_err(|_| Error::VaultNotFound)?;

        if account_info.data.is_empty() {
            return Err(Error::VaultNotFound);
        }

        let (vault_authority_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let user_token_account = get_associated_token_address(&user_pubkey, &self.mint);
        let vault_token_account = get_associated_token_address(&vault_authority_pda, &self.mint);

        let rpc_client_ata = self.rpc_client.clone();
        let user_token_account_clone = user_token_account;
        let user_ata_exists = tokio::task::spawn_blocking(move || {
            rpc_client_ata.get_account(&user_token_account_clone)
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?;

        let mut instructions = vec![];

        if user_ata_exists.is_err() {
            let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
                &self.payer.pubkey(),
                &user_pubkey,
                &self.mint,
                &spl_token::ID,
            );
            instructions.push(create_ata_ix);
        }

        let deposit_instruction = self.tx_builder.build_deposit_instruction(
            user_pubkey,
            vault_pda,
            user_token_account,
            vault_token_account,
            self.mint,
            vault_authority_pda,
            amount,
        );
        instructions.push(deposit_instruction);

        let recent_blockhash = tokio::task::spawn_blocking({
            let rpc_client = self.rpc_client.clone();
            move || rpc_client.get_latest_blockhash()
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
        .map_err(|e| Error::SolanaClient(format!("Failed to get blockhash: {}", e)))?;

        let mut transaction = Transaction::new_with_payer(
            &instructions,
            Some(&user_pubkey),
        );
        transaction.message.recent_blockhash = recent_blockhash;

        if user_pubkey == self.payer.pubkey() {
            transaction.try_sign(&[&*self.payer], recent_blockhash)
                .map_err(|e| Error::TransactionFailed(format!("Failed to sign transaction: {}", e)))?;
        } else {
            return Err(Error::TransactionFailed(
                "Deposit requires user's wallet signature. Please sign the transaction with your wallet.".to_string()
            ));
        }

        let rpc_client = self.rpc_client.clone();
        let transaction_clone = transaction.clone();
        let signature = tokio::task::spawn_blocking(move || {
            rpc_client.send_and_confirm_transaction(&transaction_clone)
        })
        .await
        .map_err(|e| Error::TransactionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| Error::TransactionFailed(format!("Failed to send transaction: {}", e)))?;

        self.database.create_transaction(
            &vault_pda.to_string(),
            crate::models::TransactionType::Deposit,
            amount,
            Some(&signature.to_string()),
        ).await?;

        Ok(signature.to_string())
    }

    pub async fn withdraw(&self, user: &str, amount: u64) -> Result<String> {
        let user_pubkey = Pubkey::from_str(user)
            .map_err(|e| Error::InvalidAccount(format!("Invalid user pubkey: {}", e)))?;

        require!(amount > 0, Error::InvalidAccount("Amount must be greater than 0".to_string()));

        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let vault_info = self.get_vault_info(user).await?;
        if vault_info.available_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let (vault_authority_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let (global_vault_authority, _) = Pubkey::find_program_address(
            &[b"vault_authority"],
            &self.program_id,
        );

        let user_token_account = get_associated_token_address(&user_pubkey, &self.mint);
        let vault_token_account = get_associated_token_address(&vault_authority_pda, &self.mint);

        let instruction = self.tx_builder.build_withdraw_instruction(
            user_pubkey,
            vault_pda,
            user_token_account,
            vault_token_account,
            self.mint,
            vault_authority_pda,
            global_vault_authority,
            amount,
        );

        let recent_blockhash = tokio::task::spawn_blocking({
            let rpc_client = self.rpc_client.clone();
            move || rpc_client.get_latest_blockhash()
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
        .map_err(|e| Error::SolanaClient(format!("Failed to get blockhash: {}", e)))?;

        let mut transaction = Transaction::new_with_payer(
            &[instruction],
            Some(&user_pubkey),
        );
        transaction.message.recent_blockhash = recent_blockhash;

        if user_pubkey == self.payer.pubkey() {
            transaction.try_sign(&[&*self.payer], recent_blockhash)
                .map_err(|e| Error::TransactionFailed(format!("Failed to sign transaction: {}", e)))?;
        } else {
            return Err(Error::TransactionFailed(
                "Withdraw requires user's wallet signature. Please sign the transaction with your wallet.".to_string()
            ));
        }

        let rpc_client = self.rpc_client.clone();
        let transaction_clone = transaction.clone();
        let signature = tokio::task::spawn_blocking(move || {
            rpc_client.send_and_confirm_transaction(&transaction_clone)
        })
        .await
        .map_err(|e| Error::TransactionFailed(format!("Task join error: {}", e)))?
        .map_err(|e| Error::TransactionFailed(format!("Failed to send transaction: {}", e)))?;

        self.database.create_transaction(
            &vault_pda.to_string(),
            crate::models::TransactionType::Withdrawal,
            amount,
            Some(&signature.to_string()),
        ).await?;

        Ok(signature.to_string())
    }

    pub async fn get_vault_info(&self, user: &str) -> Result<VaultInfo> {
        let user_pubkey = Pubkey::from_str(user)
            .map_err(|e| Error::InvalidAccount(format!("Invalid user pubkey: {}", e)))?;

        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", user_pubkey.as_ref()],
            &self.program_id,
        );

        let rpc_client = self.rpc_client.clone();
        let vault_pda_clone = vault_pda;
        let account_info = tokio::task::spawn_blocking(move || {
            rpc_client.get_account(&vault_pda_clone)
        })
        .await
        .map_err(|e| Error::SolanaClient(format!("Task join error: {}", e)))?
        .map_err(|e| Error::SolanaClient(format!("Failed to fetch vault account: {}", e)))?;

        if account_info.data.is_empty() {
            return Err(Error::InvalidAccount("Vault account does not exist".to_string()));
        }

        let account_data = &account_info.data[8..];
        
        if account_data.len() < 105 {
            return Err(Error::SolanaClient("Invalid account data length".to_string()));
        }

        let mut offset = 0;
        
        let owner = Pubkey::try_from(&account_data[offset..offset+32])
            .map_err(|_| Error::SolanaClient("Failed to parse owner".to_string()))?;
        offset += 32;
        
        let token_account = Pubkey::try_from(&account_data[offset..offset+32])
            .map_err(|_| Error::SolanaClient("Failed to parse token_account".to_string()))?;
        offset += 32;
        
        let total_balance = u64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse total_balance".to_string()))?
        );
        offset += 8;
        
        let locked_balance = u64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse locked_balance".to_string()))?
        );
        offset += 8;
        
        let available_balance = u64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse available_balance".to_string()))?
        );
        offset += 8;
        
        let total_deposited = u64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse total_deposited".to_string()))?
        );
        offset += 8;
        
        let total_withdrawn = u64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse total_withdrawn".to_string()))?
        );
        offset += 8;
        
        let created_at = i64::from_le_bytes(
            account_data[offset..offset+8].try_into()
                .map_err(|_| Error::SolanaClient("Failed to parse created_at".to_string()))?
        );
        offset += 8;
        
        let _bump = account_data[offset];

        Ok(VaultInfo {
            owner: owner.to_string(),
            vault: vault_pda.to_string(),
            token_account: token_account.to_string(),
            total_balance,
            locked_balance,
            available_balance,
            total_deposited,
            total_withdrawn,
            created_at,
        })
    }
}
