use anchor_lang::prelude::*;
use collateral_vault::cpi;
use collateral_vault::cpi::accounts::{LockCollateral, TransferCollateral, UnlockCollateral};
use collateral_vault::program::CollateralVault;

declare_id!("81AHUA8wqbBcHB4VHJztGvG3fD8LncBzvY8YqWdMSnVv");

#[program]
pub mod test_caller {
    use super::*;

    pub fn lock(ctx: Context<LockViaCpi>, amount: u64) -> Result<()> {
        let seeds = &[b"cpi_authority".as_ref(), &[ctx.bumps.cpi_authority]];
        let signer = &[&seeds[..]];

        cpi::lock_collateral(
            CpiContext::new_with_signer(
                ctx.accounts.vault_program.to_account_info(),
                LockCollateral {
                    vault: ctx.accounts.vault.to_account_info(),
                    vault_authority: ctx.accounts.vault_authority.to_account_info(),
                    caller_program: ctx.accounts.caller_program.to_account_info(),
                    cpi_authority: ctx.accounts.cpi_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )
    }

    pub fn unlock(ctx: Context<UnlockViaCpi>, amount: u64) -> Result<()> {
        let seeds = &[b"cpi_authority".as_ref(), &[ctx.bumps.cpi_authority]];
        let signer = &[&seeds[..]];

        cpi::unlock_collateral(
            CpiContext::new_with_signer(
                ctx.accounts.vault_program.to_account_info(),
                UnlockCollateral {
                    vault: ctx.accounts.vault.to_account_info(),
                    vault_authority: ctx.accounts.vault_authority.to_account_info(),
                    caller_program: ctx.accounts.caller_program.to_account_info(),
                    cpi_authority: ctx.accounts.cpi_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )
    }

    pub fn transfer(ctx: Context<TransferViaCpi>, amount: u64) -> Result<()> {
        let seeds = &[b"cpi_authority".as_ref(), &[ctx.bumps.cpi_authority]];
        let signer = &[&seeds[..]];

        cpi::transfer_collateral(
            CpiContext::new_with_signer(
                ctx.accounts.vault_program.to_account_info(),
                TransferCollateral {
                    from_vault: ctx.accounts.from_vault.to_account_info(),
                    to_vault: ctx.accounts.to_vault.to_account_info(),
                    from_vault_token_account: ctx
                        .accounts
                        .from_vault_token_account
                        .to_account_info(),
                    to_vault_token_account: ctx.accounts.to_vault_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to_vault_authority: ctx.accounts.to_vault_authority.to_account_info(),
                    from_vault_authority: ctx.accounts.from_vault_authority.to_account_info(),
                    vault_authority: ctx.accounts.vault_authority.to_account_info(),
                    caller_program: ctx.accounts.caller_program.to_account_info(),
                    cpi_authority: ctx.accounts.cpi_authority.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer,
            ),
            amount,
        )
    }
}

#[derive(Accounts)]
pub struct LockViaCpi<'info> {
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub vault: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: this program
    #[account(address = crate::ID)]
    pub caller_program: AccountInfo<'info>,
    /// CHECK: PDA signed by this program
    #[account(seeds = [b"cpi_authority"], bump)]
    pub cpi_authority: AccountInfo<'info>,
    pub vault_program: Program<'info, CollateralVault>,
}

#[derive(Accounts)]
pub struct UnlockViaCpi<'info> {
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub vault: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: this program
    #[account(address = crate::ID)]
    pub caller_program: AccountInfo<'info>,
    /// CHECK: PDA signed by this program
    #[account(seeds = [b"cpi_authority"], bump)]
    pub cpi_authority: AccountInfo<'info>,
    pub vault_program: Program<'info, CollateralVault>,
}

#[derive(Accounts)]
pub struct TransferViaCpi<'info> {
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub from_vault: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub to_vault: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub from_vault_token_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: vault program validates this account
    pub to_vault_token_account: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub mint: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub to_vault_authority: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub from_vault_authority: AccountInfo<'info>,
    /// CHECK: vault program validates this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: this program
    #[account(address = crate::ID)]
    pub caller_program: AccountInfo<'info>,
    /// CHECK: PDA signed by this program
    #[account(seeds = [b"cpi_authority"], bump)]
    pub cpi_authority: AccountInfo<'info>,
    /// CHECK: token program
    pub token_program: AccountInfo<'info>,
    pub vault_program: Program<'info, CollateralVault>,
}
