#![allow(unused_variables)]

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    instruction::Initialize,
    state::Config,
    utils::{check_pda_and_get_bump, create_mint, create_token_account},
};

pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let initialize = Initialize::try_from(data)?;
    let seed = initialize.seed;
    let fee = initialize.fee;
    let authority = initialize.authority;
    let padding = initialize.padding;

    let [initializer, mint_x, mint_y, mint_lp, vault_x, vault_y, config, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let x_bump = check_pda_and_get_bump(
        &[mint_x.key.as_ref(), config.key.as_ref()],
        &crate::ID,
        vault_x.key,
    )?;

    let y_bump = check_pda_and_get_bump(
        &[mint_y.key.as_ref(), config.key.as_ref()],
        &crate::ID,
        vault_y.key,
    )?;

    let lp_bump = check_pda_and_get_bump(&[config.key.as_ref()], &crate::ID, mint_lp.key)?;

    Config::initialize(
        seed,
        authority,
        fee,
        lp_bump,
        x_bump,
        y_bump,
        mint_x,
        mint_y,
        initializer,
        config,
    )?;

    assert_eq!(spl_token::ID, *token_program.key);

    // Create the x_vault
    create_token_account(
        &[mint_x.key.as_ref(), config.key.as_ref(), &[x_bump]],
        token_program.key,
        initializer,
        vault_x,
        mint_x,
        config,
    )?;

    // Create the y_vault
    create_token_account(
        &[mint_y.key.as_ref(), config.key.as_ref(), &[y_bump]],
        token_program.key,
        initializer,
        vault_y,
        mint_y,
        config,
    )?;

    // Create the lp_mint
    create_mint(
        &[config.key.as_ref(), &[lp_bump]],
        token_program.key,
        initializer,
        mint_lp,
        config,
    )
}
