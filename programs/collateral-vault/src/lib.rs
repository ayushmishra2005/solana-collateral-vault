use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("Fwfy9VwuzRqhQoCh4pk9JJ3dpBdTipUMPLVByCLWp6hf");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        vault.owner = ctx.accounts.user.key();
        vault.token_account = ctx.accounts.vault_token_account.key();
        vault.total_balance = 0;
        vault.locked_balance = 0;
        vault.available_balance = 0;
        vault.total_deposited = 0;
        vault.total_withdrawn = 0;
        vault.created_at = clock.unix_timestamp;
        vault.bump = ctx.bumps.vault;

        emit!(VaultInitialized {
            user: ctx.accounts.user.key(),
            vault: vault.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Transfer USDT from user to vault using CPI
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // Update vault state
        let vault = &mut ctx.accounts.vault;
        vault.total_balance = vault
            .total_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        vault.available_balance = vault
            .available_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        vault.total_deposited = vault
            .total_deposited
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        let clock = Clock::get()?;
        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            vault: vault.key(),
            amount,
            new_balance: vault.total_balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        require!(
            vault.available_balance >= amount,
            ErrorCode::InsufficientAvailableBalance
        );

        // Transfer USDT from vault to user using CPI
        let seeds = &[
            b"vault",
            vault.owner.as_ref(),
            &[vault.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_authority_pda.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        // Update vault state
        vault.total_balance = vault
            .total_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;
        vault.available_balance = vault
            .available_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;
        vault.total_withdrawn = vault
            .total_withdrawn
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        let clock = Clock::get()?;
        emit!(WithdrawEvent {
            user: ctx.accounts.user.key(),
            vault: vault.key(),
            amount,
            new_balance: vault.total_balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        require!(
            vault.available_balance >= amount,
            ErrorCode::InsufficientAvailableBalance
        );

        // Verify caller is authorized program
        let vault_authority = &ctx.accounts.vault_authority;
        require!(
            vault_authority.authorized_programs.contains(&ctx.accounts.caller_program.key()),
            ErrorCode::UnauthorizedProgram
        );

        vault.locked_balance = vault
            .locked_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        vault.available_balance = vault
            .available_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        let clock = Clock::get()?;
        emit!(LockEvent {
            user: vault.owner,
            vault: vault.key(),
            amount,
            locked_balance: vault.locked_balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        require!(
            vault.locked_balance >= amount,
            ErrorCode::InsufficientLockedBalance
        );

        // Verify caller is authorized program
        let vault_authority = &ctx.accounts.vault_authority;
        require!(
            vault_authority.authorized_programs.contains(&ctx.accounts.caller_program.key()),
            ErrorCode::UnauthorizedProgram
        );

        vault.locked_balance = vault
            .locked_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;
        vault.available_balance = vault
            .available_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        let clock = Clock::get()?;
        emit!(UnlockEvent {
            user: vault.owner,
            vault: vault.key(),
            amount,
            locked_balance: vault.locked_balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn transfer_collateral(
        ctx: Context<TransferCollateral>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidAmount);

        let from_vault = &mut ctx.accounts.from_vault;
        let to_vault = &mut ctx.accounts.to_vault;

        require!(
            from_vault.available_balance >= amount,
            ErrorCode::InsufficientAvailableBalance
        );

        // Verify caller is authorized program
        let vault_authority = &ctx.accounts.vault_authority;
        require!(
            vault_authority.authorized_programs.contains(&ctx.accounts.caller_program.key()),
            ErrorCode::UnauthorizedProgram
        );

        // Transfer tokens between vaults
        let seeds = &[
            b"vault",
            from_vault.owner.as_ref(),
            &[from_vault.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from_vault_token_account.to_account_info(),
                    to: ctx.accounts.to_vault_token_account.to_account_info(),
                    authority: ctx.accounts.from_vault_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        // Update from_vault state
        from_vault.total_balance = from_vault
            .total_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;
        from_vault.available_balance = from_vault
            .available_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        // Update to_vault state
        to_vault.total_balance = to_vault
            .total_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        to_vault.available_balance = to_vault
            .available_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        let clock = Clock::get()?;
        emit!(TransferEvent {
            from_user: from_vault.owner,
            to_user: to_vault.owner,
            from_vault: from_vault.key(),
            to_vault: to_vault.key(),
            amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn initialize_vault_authority(
        ctx: Context<InitializeVaultAuthority>,
        authorized_programs: Vec<Pubkey>,
    ) -> Result<()> {
        let vault_authority = &mut ctx.accounts.vault_authority;
        vault_authority.authorized_programs = authorized_programs;
        vault_authority.bump = ctx.bumps.vault_authority;

        Ok(())
    }
}

#[account]
pub struct CollateralVault {
    pub owner: Pubkey,
    pub token_account: Pubkey,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct VaultAuthority {
    pub authorized_programs: Vec<Pubkey>,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + CollateralVault::LEN,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = vault_authority_pda
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    /// CHECK: PDA authority for vault token account
    pub vault_authority_pda: AccountInfo<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault_authority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump
    )]
    /// CHECK: PDA authority for vault token account
    pub vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key() @ ErrorCode::UnauthorizedOwner
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault_authority_pda
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump
    )]
    /// CHECK: PDA authority for vault token account
    pub vault_authority_pda: AccountInfo<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct LockCollateral<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: Verified by checking authorized_programs
    pub caller_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UnlockCollateral<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: Verified by checking authorized_programs
    pub caller_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    #[account(
        mut,
        seeds = [b"vault", from_vault.owner.as_ref()],
        bump = from_vault.bump,
    )]
    pub from_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault", to_vault.owner.as_ref()],
        bump = to_vault.bump,
    )]
    pub to_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = from_vault_authority
    )]
    pub from_vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = to_vault_authority
    )]
    pub to_vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        seeds = [b"vault", to_vault.owner.as_ref()],
        bump = to_vault.bump
    )]
    /// CHECK: PDA authority for to_vault
    pub to_vault_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"vault", from_vault.owner.as_ref()],
        bump = from_vault.bump
    )]
    /// CHECK: PDA authority for from_vault
    pub from_vault_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: Verified by checking authorized_programs
    pub caller_program: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeVaultAuthority<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + VaultAuthority::LEN,
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    pub system_program: Program<'info, System>,
}

impl CollateralVault {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}

impl VaultAuthority {
    pub const LEN: usize = 4 + (32 * 10) + 1; // Vec<Pubkey> with max 10 programs + bump
}

#[event]
pub struct VaultInitialized {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct LockEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub locked_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct UnlockEvent {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub locked_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct TransferEvent {
    pub from_user: Pubkey,
    pub to_user: Pubkey,
    pub from_vault: Pubkey,
    pub to_vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient available balance")]
    InsufficientAvailableBalance,
    #[msg("Insufficient locked balance")]
    InsufficientLockedBalance,
    #[msg("Unauthorized owner")]
    UnauthorizedOwner,
    #[msg("Unauthorized program")]
    UnauthorizedProgram,
    #[msg("Integer overflow")]
    Overflow,
    #[msg("Integer underflow")]
    Underflow,
}
