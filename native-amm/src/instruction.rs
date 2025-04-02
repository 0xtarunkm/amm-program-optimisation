use amm_macros::TryFromBytes;
use bytemuck::{Pod, Zeroable};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(Clone, Copy)]
pub enum AmmInstructions {
    Initialize,
    Deposit,
    Swap,
}

impl TryFrom<&u8> for AmmInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Deposit),
            3 => Ok(Self::Swap),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Initialize {
    pub seed: u64,
    pub fee: u16,
    pub authority: Pubkey,
    pub padding: [u8; 6],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Deposit {
    pub amount: u64,
    pub max_x: u64,
    pub max_y: u64,
    pub expiration: i64,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable, TryFromBytes)]
pub struct Swap {
    pub amount: u64,
    pub min: u64,
    pub expiration: i64,
}
