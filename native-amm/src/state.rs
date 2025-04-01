use amm_macros::TryFromBytes;
use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction::create_account;
use solana_program::sysvar::Sysvar;

use crate::utils::check_pda_and_get_bump;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, TryFromBytes)]
pub struct Config {
    pub seed: u64,
    pub authority: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16,
    pub locked: u8,
    pub config_bump: u8,
    pub lp_bump: u8,
    pub x_bump: u8,
    pub y_bump: u8,
    pub padding: [u8; 1],
}

impl Config {
    pub fn initialize<'a>(
        seed: u64,
        authority: Pubkey,
        fee: u16,
        lp_bump: u8,
        x_bump: u8,
        y_bump: u8,
        mint_x: &AccountInfo,
        mint_y: &AccountInfo,
        initializer: &AccountInfo<'a>,
        config: &AccountInfo<'a>,
    ) -> ProgramResult {
        let config_bump = check_pda_and_get_bump(
            &[b"config", seed.to_le_bytes().as_ref()],
            &crate::ID,
            config.key,
        )?;

        assert!(fee < 10_000);

        let _ = spl_token::state::Mint::unpack(&mint_x.try_borrow_data()?);
        let _ = spl_token::state::Mint::unpack(&mint_y.try_borrow_data()?);

        let config_space = core::mem::size_of::<Config>();
        let config_rent = Rent::get()?.minimum_balance(config_space);

        invoke_signed(
            &create_account(
                initializer.key,
                config.key,
                config_rent,
                config_space as u64,
                &crate::ID,
            ),
            &[initializer.clone(), config.clone()],
            &[&[b"config", seed.to_le_bytes().as_ref(), &[config_bump]]],
        )?;

        let mut config_data: Config =
            *bytemuck::try_from_bytes_mut::<Config>(*config.data.borrow_mut())
                .map_err(|_| ProgramError::InvalidAccountData)?;

        config_data.clone_from(&Config {
            seed,
            authority,
            mint_x: *mint_x.key,
            mint_y: *mint_y.key,
            fee,
            locked: 0,
            config_bump,
            lp_bump,
            x_bump,
            y_bump,
            padding: [0; 1],
        });

        Ok(())
    }
}
