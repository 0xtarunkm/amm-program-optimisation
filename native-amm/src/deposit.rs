#![allow(unused_variables)]
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{instruction::Deposit, state::Config};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let deposit = Deposit::try_from(data)?;
    let amount = deposit.amount;
    let max_x = deposit.max_x;
    let max_y = deposit.max_y;
    let expiration = deposit.expiration;

    let [user, mint_x, mint_y, mint_lp, user_x, user_y, user_lp, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(user.is_signer);

    assert_eq!(token_program.key, &spl_token::ID);

    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    Config::add_liquidity(
        amount,
        max_x,
        max_y,
        &config_account,
        token_program.key,
        user_x,
        user_y,
        user_lp,
        vault_x,
        vault_y,
        mint_x,
        mint_y,
        mint_lp,
        config,
        user,
    )
}
