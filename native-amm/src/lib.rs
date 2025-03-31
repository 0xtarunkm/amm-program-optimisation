use instruction::AmmInstructions;
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

mod instruction;
mod deposit;
mod swap;
mod initialize;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    let (tag, rest) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match AmmInstructions::try_from(tag)? {
        AmmInstructions::Initialize => initialize::process(accounts, rest),
        AmmInstructions::Deposit => deposit::process(accounts, rest),
        AmmInstructions::Swap => swap::process(accounts, rest)
    }
    
}