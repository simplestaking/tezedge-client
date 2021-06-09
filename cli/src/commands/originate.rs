use std::thread;
use std::fmt::{self, Display};
use std::time::Duration;
use cli_spinner::SpinnerBuilder;
use structopt::StructOpt;
use console::style;

use lib::{ToBase58Check, Forge, Forged, KeyDerivationPath, MANAGER_CONTRACT_CODE};
use lib::utils::parse_float_amount;
use lib::{ImplicitAddress, NewOperationGroup, NewOriginationOperation, NewOriginationScript};
use lib::micheline::Micheline;
use lib::signer::OperationSignatureInfo;
use lib::api::*;
use lib::http_api::HttpApi;
use lib::trezor_api::{Trezor, TezosSignTx};
use lib::ledger_api::Ledger;
use lib::crypto::hex;

use crate::trezor::trezor_execute;
use crate::ledger::ledger_execute;
use crate::common::operation_command::*;
use crate::commands::CommandError;

#[derive(thiserror::Error, Debug)]
pub struct InvalidBalanceError(pub String);

impl Display for InvalidBalanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid balance: {}", style(&self.0).red())
    }
}

struct State {
    address: Option<ImplicitAddress>,
    counter: Option<u64>,
    trezor: Option<Trezor>,
    ledger: Option<Ledger>,
}

impl Default for State {
    fn default() -> Self {
        State {
            address: None,
            counter: None,
            trezor: None,
            ledger: None,
        }
    }
}

/// Delegate balance to baker
#[derive(StructOpt)]
pub struct Originate {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Disable interactivity and accept default answers to prompts.
    #[structopt(short = "y", long = "no-prompt")]
    pub no_prompt: bool,

    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "trezor")]
    pub use_trezor: bool,

    #[structopt(long = "ledger")]
    pub use_ledger: bool,

    #[structopt(long = "key-path")]
    pub key_path: String,

    #[structopt(long)]
    pub balance: String,

    #[structopt(long)]
    pub fee: String,

    #[structopt(skip)]
    state: State,
}

impl Originate {
    pub fn get_endpoint(&self) -> Option<&str> {
        Some(self.endpoint.as_str())
    }

    fn rpc(&self) -> HttpApi {
        HttpApi::new(self.endpoint.clone())
    }

    fn key_path(&self) -> Result<KeyDerivationPath, CommandError> {
        Ok(self.key_path.parse()?)
    }

    fn balance(&self) -> Result<u64, CommandError> {
        Ok(parse_float_amount(&self.balance)
            .map_err(|_| InvalidBalanceError(self.balance.clone()))?)
    }

    fn fee(&self) -> Result<u64, InvalidFeeError> {
        Ok(parse_float_amount(&self.fee)
            .map_err(|_| InvalidFeeError(self.fee.clone()))?)
    }

    fn get_counter(&mut self) -> Result<u64, GetContractCounterError> {
        let counter = self.state.counter
            .map(|value| Ok(value))
            .unwrap_or_else(|| {
                self.rpc().get_contract_counter(&self.address().unwrap().into())
            })? + 1;

        self.state.counter.replace(counter);
        Ok(counter)
    }

    fn trezor(&mut self) -> &mut Trezor {
        self.state.trezor.get_or_insert_with(|| crate::trezor::find_device_and_connect())
    }

    fn ledger(&mut self) -> &mut Ledger {
        self.state.ledger.get_or_insert_with(|| crate::ledger::find_device_and_connect())
    }

    fn address(&mut self) -> Result<ImplicitAddress, CommandError> {
        if let Some(address) = self.state.address.as_ref() {
            Ok(address.clone())
        } else {
            let key_path = self.key_path()?;

            let address = if self.use_trezor {
                crate::trezor::get_address(
                    self.trezor(),
                    &key_path,
                )
            } else if self.use_ledger {
                ledger_execute(
                    self.ledger().get_address(&key_path, false),
                )
            } else {
                unreachable!()
            };

            self.state.address = Some(address.clone());
            Ok(address)
        }

    }

    fn build_operation_group(&mut self) -> Result<NewOperationGroup, CommandError> {
        let address = self.address()?;
        let operation_group = NewOperationGroup::new(
            self.rpc().get_head_block_hash()?,
            self.rpc().get_protocol_info()?.next_protocol_hash,
        );

        Ok(operation_group.with_origination(NewOriginationOperation {
            source: address.clone(),
            balance: self.balance()?,
            fee: self.fee()?,
            counter: self.get_counter()?,
            gas_limit: 10000,
            storage_limit: 10000,
            script: NewOriginationScript {
                // use hardcoded manager.tz script code.
                code: Forged::new_unchecked(hex::decode(MANAGER_CONTRACT_CODE).unwrap()),
                storage: Micheline::String(address.to_base58check()),
            }
        }))
    }

    fn sign_operation(&mut self, op_group: &NewOperationGroup) -> Result<OperationSignatureInfo, CommandError> {
        let key_path = self.key_path()?;
        Ok(if self.use_trezor {
            let mut tx: TezosSignTx = op_group.clone().into();
            tx.set_address_n(key_path.take());
            OperationSignatureInfo::from(
                trezor_execute(self.trezor().sign_tx(tx))
            )
        } else {
            ledger_execute(
                self.ledger().sign_tx(&key_path, op_group.forge())
            )
        })
    }

    fn confirm_operation(&mut self, operation_hash: &str) -> Result<(), CommandError> {
        let spinner = SpinnerBuilder::new()
            .with_text("waiting for confirmation")
            .start();

        let api = self.rpc();
        for _ in 0..10 {
            thread::sleep(Duration::from_secs(2));

            let status = api.get_pending_operation_status(&operation_hash)?;
            match status {
                PendingOperationStatus::Refused => {
                    spinner.finish_fail("operation_refused");
                    return Ok(());
                }
                PendingOperationStatus::Applied => {
                }
                PendingOperationStatus::Finished => {
                    break;
                }
            }
        }

        spinner.finish_succeed("operation confirmed");

        Ok(())
    }

    pub fn execute(mut self) -> Result<(), CommandError> {
        let op_group = self.build_operation_group()?;
        let OperationSignatureInfo {
            operation_hash,
            operation_with_signature,
            ..
        } = self.sign_operation(&op_group)?;

        self.rpc().inject_operations(&operation_with_signature)?;

        self.confirm_operation(&operation_hash)?;

        println!("{}", &operation_hash);

        Ok(())
    }
}
