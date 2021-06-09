use structopt::StructOpt;

pub mod address;
pub mod transfer;
pub mod delegate;
pub mod originate;

// These two are temporary, before local store will be implemented.
pub mod transfer_local;
pub mod delegate_local;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(StructOpt)]
pub enum Command {
    Address(address::Address),
    Transfer(transfer::Transfer),
    Delegate(delegate::Delegate),
    TransferLocal(transfer_local::TransferLocal),
    DelegateLocal(delegate_local::DelegateLocal),
    #[structopt(setting(structopt::clap::AppSettings::Hidden))]
    Originate(originate::Originate),
}

impl Command {
    /// Get node endpoint.
    pub fn get_endpoint(&self) -> Option<&str> {
        match self {
            Self::Address(cmd) => cmd.get_endpoint(),
            Self::Transfer(cmd) => cmd.get_endpoint(),
            Self::Delegate(cmd) => cmd.get_endpoint(),
            Self::TransferLocal(cmd) => cmd.get_endpoint(),
            Self::DelegateLocal(cmd) => cmd.get_endpoint(),
            Self::Originate(cmd) => cmd.get_endpoint(),
        }
    }
}
