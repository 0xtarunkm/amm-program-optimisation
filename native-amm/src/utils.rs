use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::instruction::{initialize_account3, initialize_mint2};

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
