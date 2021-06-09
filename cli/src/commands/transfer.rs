use structopt::StructOpt;
use console::style;

use lib::Address;
use lib::utils::parse_float_amount;
use crate::commands::CommandError;
use crate::common::exit_with_error;
use crate::common::operation_command::*;

/// Create a transaction.
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt)]
pub struct Transfer {
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

    /// Address to transfer tezos from.
    ///
    /// When transfering from scriptless smart contract (KT1) address,
    /// this needs to be that KT1 address.
    ///
    /// Otherwise use key derivation path, like: "m/44'/1729'/0'/0'"
    #[structopt(short, long)]
    pub from: String,

    /// Address to transfer funds to.
    #[structopt(short, long)]
    pub to: String,

    /// Amount to transfer.
    #[structopt(short, long)]
    pub amount: String,

    /// Fee for the transaction.
    ///
    /// If not specified, fee will be estimated and you will be prompted
    /// whether or not you accept estimate or would like to enter custom one.
    #[structopt(long)]
    pub fee: Option<String>,
}

impl RawOperationCommand for Transfer {
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

impl Transfer {
    pub fn get_endpoint(&self) -> Option<&str> {
        Some(self.endpoint.as_str())
    }

    fn get_amount(&self) -> u64 {
        match parse_float_amount(&self.amount) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid amount: {}",
                    style(&self.amount).bold()
                ));
            }
        }
    }

    pub fn execute(self) -> Result<(), CommandError> {
        let to = Address::from_base58check(&self.to)
           .map_err(|err| ParseAddressError {
               kind: AddressKind::Destination,
               error: err,
               address: self.to.clone(),
           })?;
        Ok(self.parse()?.transfer(to, self.get_amount())?)
    }
}
