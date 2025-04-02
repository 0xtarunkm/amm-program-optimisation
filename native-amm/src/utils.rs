use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::instruction::{
    initialize_account3, initialize_mint2, mint_to_checked, transfer_checked,
};

use crate::state::Config;

#[inline]
pub fn check_pda_with_bump(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<(), ProgramError> {
    let derived_address = Pubkey::create_program_address(seeds, program_id)?;
    Ok(assert!(derived_address.eq(address)))
}

#[inline]
pub fn check_pda_and_get_bump(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<u8, ProgramError> {
    let (derived_address, bump) = Pubkey::try_find_program_address(seeds, program_id)
        .ok_or(ProgramError::InvalidAccountData)?;
    assert!(derived_address.eq(address));
    Ok(bump)
}

pub fn create_token_account<'a>(
    seeds: &[&[u8]],
    token_program: &Pubkey,
    payer: &AccountInfo<'a>,
    ta: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
) -> ProgramResult {
    let token_space = spl_token::state::Account::LEN;
    let token_rent = Rent::get()?.minimum_balance(token_space);

    invoke_signed(
        &create_account(
            payer.key,
            ta.key,
            token_rent,
            token_space as u64,
            &spl_token::ID,
        ),
        &[payer.clone(), ta.clone()],
        &[seeds],
    )?;

    invoke(
        &initialize_account3(token_program, ta.key, mint.key, authority.key)?,
        &[ta.clone(), mint.clone()],
    )
}

pub fn create_mint<'a>(
    seeds: &[&[u8]],
    token_program: &Pubkey,
    payer: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
) -> ProgramResult {
    let mint_space = spl_token::state::Mint::LEN;
    let mint_rent = Rent::get()?.minimum_balance(mint_space);

    invoke_signed(
        &create_account(
            payer.key,
            mint.key,
            mint_rent,
            mint_space as u64,
            &spl_token::ID,
        ),
        &[payer.clone(), mint.clone()],
        &[seeds],
    )?;

    invoke(
        &initialize_mint2(token_program, mint.key, authority.key, None, 0)?,
        &[mint.clone()],
    )
}

#[inline]
pub fn perform_basic_checks(
    config_account: &Config,
    expiration: i64,
    config: &AccountInfo,
    mint_lp: &AccountInfo,
    vault_x: &AccountInfo,
    vault_y: &AccountInfo,
) -> ProgramResult {
    assert!(Clock::get()?.unix_timestamp <= expiration);

    assert_eq!(config.owner, &crate::ID);

    assert_ne!(config_account.locked, 1);

    check_pda_with_bump(
        &[config.key.as_ref(), &[config_account.lp_bump]],
        &crate::ID,
        mint_lp.key,
    )?;

    check_pda_with_bump(
        &[
            config_account.mint_x.as_ref(),
            config.key.as_ref(),
            &[config_account.x_bump],
        ],
        &crate::ID,
        vault_x.key,
    )?;

    check_pda_with_bump(
        &[
            config_account.mint_y.as_ref(),
            config.key.as_ref(),
            &[config_account.y_bump],
        ],
        &crate::ID,
        vault_y.key,
    )?;

    Ok(())
}

#[inline]
pub fn deposit<'a>(
    token_program: &Pubkey,
    user_from: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    vault: &AccountInfo<'a>,
    user: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
) -> ProgramResult {
    invoke(
        &transfer_checked(
            token_program,
            user_from.key,
            mint.key,
            vault.key,
            user.key,
            &[],
            amount,
            decimals,
        )?,
        &[user_from.clone(), mint.clone(), vault.clone(), user.clone()],
    )
}

#[inline]
pub fn mint<'a>(
    token_program: &Pubkey,
    mint: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    seeds: &[&[u8]],
) -> ProgramResult {
    // Transfer the funds from the maker's token account to the vault
    invoke_signed(
        &mint_to_checked(
            token_program,
            mint.key,
            to.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[mint.clone(), to.clone(), authority.clone()],
        &[seeds],
    )
}

#[inline]
pub fn withdraw<'a>(
    token_program: &Pubkey,
    vault: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    user: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &transfer_checked(
            token_program,
            vault.key,
            mint.key,
            user.key,
            authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[vault.clone(), mint.clone(), user.clone(), authority.clone()],
        &[seeds],
    )
}
