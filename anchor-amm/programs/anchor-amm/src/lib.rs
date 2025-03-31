use anchor_lang::prelude::*;
mod errors;
mod instructions;
mod state;
use instructions::*;
declare_id!("GpjB8kfUpEifuQxtRBsZYHr5nMchzCYYE3Hj3UJFnaun");

#[program]
pub mod anchor_amm {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>, seed: u64, fee: u16) -> Result<()> {
        ctx.accounts.init(seed, fee, &ctx.bumps)
    }

    pub fn add_liquidity(ctx: Context<Deposit>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64, from_x: bool) -> Result<()> {
        ctx.accounts.swap(amount_in, min_amount_out, from_x)
    }
}
