use crate::BlockRef;
use thiserror::Error;

pub trait Validate {
    fn validate(&self) -> Result<(), ValidateErr>;
}

pub trait OpValidator {
    fn validate_op(&self) -> Result<(), ValidateErr>;
}

#[derive(Debug, Error)]
pub enum ValidateErr {
    #[error("Region contains no block with name '{0}'")]
    BlockNotRegisteredWithRegion(String),
    #[error("The last operation in basic block must be a terminator")]
    BlockMissingTerminator(BlockRef),
}
