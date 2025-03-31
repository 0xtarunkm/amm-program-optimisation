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
}
