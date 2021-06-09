use structopt::StructOpt;
use console::style;

use lib::ImplicitAddress;

use crate::commands::CommandError;
use crate::common::exit_with_error;
use crate::common::operation_command::*;

/// Delegate balance to baker.
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt)]
pub struct Delegate {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Disable interactivity and accept default answers to prompts.
    #[structopt(short = "y", long = "no-prompt")]
    pub no_prompt: bool,

    /// Node's rpc endpoint.
    ///
    /// Sample Testnet nodes:
    /// - https://api.tez.ie/rpc/edonet
    /// - https://rpctest.tzbeta.net
    /// - https://testnet-tezos.giganode.io
    #[structopt(short = "E", long)]
    pub endpoint: String,

    /// Use Trezor device.
    #[structopt(long = "trezor")]
    pub use_trezor: bool,

    /// Use Ledger device.
    #[structopt(long = "ledger")]
    pub use_ledger: bool,

    /// Key Derivation Path.
    ///
    /// Required only when transferring/delegating from scriptless
    /// smart contract (when --from address starts with KT1).
    ///
    /// Otherwise pass key derivation path to --from argument instead!
    #[structopt(long = "key-path")]
    pub key_path: Option<String>,

    /// Address to delegate tezos from.
    ///
    /// When delegating from scriptless smart contract (KT1) address,
    /// this needs to be that KT1 address.
    ///
    /// Otherwise use key derivation path, like: "m/44'/1729'/0'/0'"
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

    /// Fee for the delegation.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,
}

impl RawOperationCommand for Delegate {
    fn get_raw_options(&self) -> RawOptions {
        RawOptions {
            api_type: "http".to_string(),
            no_prompt: self.no_prompt,
            use_trezor: self.use_trezor,
            use_ledger: self.use_ledger,
        }
    }

    fn get_api_endpoint(&self) -> String {
        self.endpoint.clone()
    }

    fn get_raw_key_path(&self) -> Option<&str> {
        self.key_path.as_ref().map(|s| s.as_str())
    }

    fn get_raw_from(&self) -> &str {
        &self.from
    }

    fn get_raw_fee(&self) -> Option<&String> {
        self.fee.as_ref()
    }
}

impl Delegate {
    pub fn get_endpoint(&self) -> Option<&str> {
        Some(self.endpoint.as_str())
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

        Ok(self.parse()?.delegate(to)?)
    }
}
