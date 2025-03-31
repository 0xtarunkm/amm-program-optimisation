use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, mint_to},
};
use crate::state::{Config, PoolStats};
use crate::errors::AmmError;

fn isqrt(n: u128) -> u128 {
    if n <= 1 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        token::mint = mint_x,
        token::authority = user,
    )]
    pub user_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint_y,
        token::authority = user,
    )]
    pub user_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint_lp,
        token::authority = user,
    )]
    pub user_lp: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint_x,
        token::authority = config,
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = mint_y,
        token::authority = config,
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,
    pub mint_x: Box<InterfaceAccount<'info, Mint>>,
    pub mint_y: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        has_one = mint_x,
        has_one = mint_y,
        constraint = !config.locked @ AmmError::PoolLocked,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"pool_stats", config.key().as_ref()],
        bump,
    )]
    pub pool_stats: AccountLoader<'info, PoolStats>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn deposit(
    ctx: Context<Deposit>, 
    amount_x: u64, 
    amount_y: u64,
    min_lp: u64,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let user_x = &ctx.accounts.user_x;
    let user_y = &ctx.accounts.user_y;
    let user_lp = &ctx.accounts.user_lp;
    let vault_x = &ctx.accounts.vault_x;
    let vault_y = &ctx.accounts.vault_y;
    let mint_x = &ctx.accounts.mint_x;
    let mint_y = &ctx.accounts.mint_y;
    let mint_lp = &ctx.accounts.mint_lp;
    let config = &ctx.accounts.config;
    let token_program = &ctx.accounts.token_program;
    let vault_x_amount = vault_x.amount;
    let vault_y_amount = vault_y.amount;
    let lp_supply = mint_lp.supply;
    let lp_amount = if lp_supply == 0 {
        isqrt(amount_x as u128 * amount_y as u128) as u64
    } else {
        let lp_from_x = (amount_x as u128 * lp_supply as u128) / vault_x_amount as u128;
        let lp_from_y = (amount_y as u128 * lp_supply as u128) / vault_y_amount as u128;
        std::cmp::min(lp_from_x, lp_from_y) as u64
    };
    require!(lp_amount >= min_lp, AmmError::SlippageExceeded);
    
    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: user_x.to_account_info(),
                to: vault_x.to_account_info(),
                authority: user.to_account_info(),
                mint: mint_x.to_account_info(),
            },
        ),
        amount_x,
        mint_x.decimals,
    )?;
    
    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            anchor_spl::token_interface::TransferChecked {
                from: user_y.to_account_info(),
                to: vault_y.to_account_info(),
                authority: user.to_account_info(),
                mint: mint_y.to_account_info(),
            },
        ),
        amount_y,
        mint_y.decimals,
    )?;
    
    let seeds = &[
        b"config",
        &config.seed.to_le_bytes()[..],
        &[config.config_bump],
    ];
    let signer = &[&seeds[..]];
    
    mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            anchor_spl::token_interface::MintTo {
                mint: mint_lp.to_account_info(),
                to: user_lp.to_account_info(),
                authority: config.to_account_info(),
            },
            signer,
        ),
        lp_amount,
    )?;
    
    // Update pool stats
    let mut pool_stats = ctx.accounts.pool_stats.load_mut()?;
    pool_stats.last_update_time = Clock::get()?.unix_timestamp;
    
    Ok(())
}