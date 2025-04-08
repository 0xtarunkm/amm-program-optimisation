use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{instruction::Swap, state::Config, utils::perform_basic_checks};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let swap = Swap::try_from(data)?;
    let amount = swap.amount;
    let min = swap.min;
    let expiration = swap.expiration;

    let [user, mint_x, mint_y, vault_x, vault_y, user_x, user_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(user.is_signer);
    assert_eq!(token_program.key, &spl_token::ID);

    let config_account = Config::try_from(config.data.borrow().as_ref())?;

    perform_basic_checks(
        &config_account,
        expiration,
        config,
        mint_x,
        vault_x,
        vault_y,
    )?;

    Config::perform_swap(
        &config_account,
        token_program.key,
        amount,
        min,
        mint_x,
        mint_y,
        vault_x,
        vault_y,
        user_x,
        user_y,
        config,
    )
}
