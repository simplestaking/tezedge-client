use std::fmt::{self, Display};
use structopt::StructOpt;
use console::style;

use lib::utils::parse_float_amount;
use lib::http_api::HttpApi;
use lib::{ImplicitAddress, PrivateKey, PublicKey};

use crate::commands::CommandError;
use crate::common::exit_with_error;
use crate::common::operation_command::*;

#[derive(thiserror::Error, Debug)]
pub struct ParseKeyError {
    kind: KeyKind,
    /// Input address as string before parsing.
    key: String,
    error: lib::FromPrefixedBase58CheckError,
}

#[derive(PartialEq, Debug, Clone)]
pub enum KeyKind {
    Public,
    Private,
}

impl Display for ParseKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field = match self.kind {
            KeyKind::Public => "--public-key",
            KeyKind::Private => "--private-key",
        };

        write!(f,
            "invalid {}: {}",
            style(field).bold(),
            style(&self.key).red(),
        )
    }
}

/// Delegate balance to baker
#[derive(StructOpt)]
pub struct DelegateLocal {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Disable interactivity and accept default answers to prompts.
    #[structopt(short = "y", long = "no-prompt")]
    pub no_prompt: bool,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "public-key")]
    pub public_key: String,

    #[structopt(long = "private-key")]
    pub private_key: String,

    #[structopt(short, long)]
    pub from: String,

    /// Address to delegate funds to.
    ///
    /// Use --cancel argument instead, to cancel active delegation.
    #[structopt(short, long)]
    pub to: Option<String>,

    /// Cancel active delegation.
    #[structopt(long)]
    pub cancel: bool,

    /// Specify fee for the delegation.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,
}

impl DelegateLocal {
    pub fn get_endpoint(&self) -> Option<&str> {
        Some(self.endpoint.as_str())
    }

    fn public_key(&self) -> Result<PublicKey, ParseKeyError> {
        PublicKey::from_base58check(&self.public_key)
            .map_err(|error| ParseKeyError {
                error,
                kind: KeyKind::Public,
                key: self.public_key.to_string(),
            })
    }

    fn private_key(&self) -> Result<PrivateKey, ParseKeyError> {
        PrivateKey::from_base58check(&self.private_key)
            .map_err(|error| ParseKeyError {
                error,
                kind: KeyKind::Private,
                key: self.private_key.to_string(),
            })
    }

    fn fee(&self) -> Result<Option<u64>, InvalidFeeError> {
        if let Some(raw_fee) = self.fee.as_ref() {
            Ok(Some(parse_float_amount(raw_fee)
                .map_err(|_| InvalidFeeError(raw_fee.to_string()))?))
        } else {
            Ok(None)
        }
    }

    pub fn execute(self) -> Result<(), CommandError> {
        if self.cancel && self.to.is_some() {
            exit_with_error(format!(
                "{} and {} can't be provided at the same time.\n\n{}\n{}",
                style("--cancel").bold(),
                style("--to").bold(),
                format!(
                    " - If you wish to {} an active delegation, please remove {} argument.",
                    style("cancel").bold(),
                    style("--to").bold(),
                ),
                format!(
                    " - If you wish to {} an active delegation, please remove {} argument.",
                    style("set").bold(),
                    style("--cancel").bold(),
                ),
            ));
        }

        if self.to.is_none() && !self.cancel {
            exit_with_error(format!(
                "Neither {} nor {} argument was supplied.\n\n{}",
                style("--to").bold(),
                style("--cancel").bold(),
                format!(
                    "Please specify either:\n{}\n{}",
                    format!(" - {} <address>, to delegate to the <address>.", style("--to").bold()),
                    format!(" - {}, to cancel active delegation.", style("--cancel").bold()),
                ),
            ));
        }

        let public_key = self.public_key()?;
        let private_key = self.private_key()?;

        let to = self.to.as_ref()
            .map(|to| {
                ImplicitAddress::from_base58check(to)
                    .map_err(|err| ParseAddressError {
                        kind: AddressKind::Destination,
                        error: err,
                        address: to.clone(),
                    })
            })
            .transpose()?;

        Ok(OperationCommand {
            options: OperationOptions {
                no_prompt: self.no_prompt,
            },
            api: Box::new(HttpApi::new(self.endpoint.clone())),
            from: public_key.hash().into(),
            fee: self.fee()?,
            state: Default::default(),
            trezor_state: None,
            ledger_state: None,
            local_state: Some(LocalWalletState { public_key, private_key }),
        }.delegate(to)?)
    }
}
