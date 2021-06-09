use structopt::StructOpt;

pub mod get_address;
pub use get_address::GetAddress;

use crate::commands::CommandError;

#[derive(StructOpt, Debug, Clone)]
pub enum Address {
    Get(GetAddress),
}

impl Address {
    pub fn get_endpoint(&self) -> Option<&str> {
        None
    }

    pub fn execute(self) -> Result<(), CommandError> {
        match self {
            Address::Get(c) => c.execute(),
        }
    }
}
