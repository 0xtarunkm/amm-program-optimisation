use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked},
};
use crate::state::Config;
use crate::errors::AmmError;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
        constraint = !config.locked @ AmmError::PoolLocked,
    )]
    pub config: Account<'info, Config>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);
        
        // Determine if we're swapping X->Y or Y->X
        let is_x_to_y = self.user_x.mint == self.mint_x.key();
        
        // Calculate the amount out based on constant product formula (x * y = k)
        let amount_out = self.calculate_amount_out(amount_in, is_x_to_y)?;
        
        // Verify the minimum output amount
        require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);
        
        // Transfer tokens from user to vault
        self.transfer_tokens_from_user(amount_in, is_x_to_y)?;
        
        // Transfer tokens from vault to user
        self.transfer_tokens_to_user(amount_out, !is_x_to_y)?;
        
        Ok(())
    }
    
    fn calculate_amount_out(
        &self,
        amount_in: u64,
        is_x_to_y: bool,
    ) -> Result<u64> {
        
        let reserve_in = if is_x_to_y { 
            self.vault_x.amount 
        } else { 
            self.vault_y.amount 
        };
        
        let reserve_out = if is_x_to_y { 
            self.vault_y.amount 
        } else { 
            self.vault_x.amount 
        };
        
        require!(reserve_in > 0 && reserve_out > 0, AmmError::InsufficientLiquidity);
        
        // Apply fee
        let fee_numerator = self.config.fee as u128;
        let fee_denominator = 10000u128;
        let amount_in_after_fee = (amount_in as u128) * (fee_denominator - fee_numerator) / fee_denominator;
        
        // Calculate amount out using constant product formula: x * y = k
        // New reserve_out = (reserve_in * reserve_out) / (reserve_in + amount_in_after_fee)
        // amount_out = Current reserve_out - New reserve_out
        let amount_out = reserve_out as u128 - 
            (reserve_in as u128 * reserve_out as u128) / 
            (reserve_in as u128 + amount_in_after_fee);
            
        let amount_out = amount_out as u64;
        
        require!(amount_out < reserve_out, AmmError::InsufficientLiquidity);
        
        Ok(amount_out)
    }
    
    fn transfer_tokens_from_user(
        &self,
        amount: u64,
        is_x: bool,
    ) -> Result<()> {
        let (from, to, mint, decimals) = if is_x {
            (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals
            )
        } else {
            (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals
            )
        };
        
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
            from,
            to,
            mint,
            authority: self.user.to_account_info(),
        };
        
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            cpi_accounts,
        );
        
        transfer_checked(ctx, amount, decimals)
    }
    
    fn transfer_tokens_to_user(
        &self,
        amount: u64,
        is_x: bool,
    ) -> Result<()> {
        let (from, to, mint, decimals) = if is_x {
            (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals
            )
        } else {
            (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals
            )
        };
        
        let seeds = &[
            b"config",
            &self.config.seed.to_le_bytes()[..],
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let cpi_accounts = anchor_spl::token_interface::TransferChecked {
            from,
            to,
            mint,
            authority: self.config.to_account_info(),
        };
        
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        
        transfer_checked(ctx, amount, decimals)
    }
}