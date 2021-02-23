use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use console::style;

use lib::{ToBase58Check, api::*, trezor_api::TezosSignTx};
use lib::{PublicKeyHash, PublicKey, PrivateKey, NewOperationGroup, NewTransactionOperationBuilder};
use lib::utils::parse_float_amount;
use lib::signer::{SignOperation, LocalSigner};

use crate::spinner::SpinnerBuilder;
use crate::common::{exit_with_error, parse_derivation_path};
use crate::emojies;
use crate::trezor::{find_trezor_device, trezor_execute};

/// Create a transaction
///
/// Outputs transaction hash to stdout in case of success.
#[derive(StructOpt, Debug, Clone)]
pub struct Transfer {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "E", long)]
    endpoint: String,

    #[structopt(long = "trezor")]
    use_trezor: bool,

    /// Address to transfer tezos from.
    ///
    /// Can either be public key hash: tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU
    ///
    /// Or if --trezor flag is set, key derivation path**: "m/44'/1729'/0'"
    #[structopt(short, long)]
    from: String,

    #[structopt(short, long)]
    to: String,

    #[structopt(short, long)]
    amount: String,

    #[structopt(long)]
    fee: String,
}

// TODO: replace with query to persistent encrypted store for keys
fn get_keys_by_pkh(pkh: &PublicKeyHash) -> Result<(PublicKey, PrivateKey), ()> {
    if pkh != &PublicKeyHash::from_base58check("tz1av5nBB8Jp6VZZDBdmGifRcETaYc7UkEnU").unwrap() {
        return Err(());
    }
    let pub_key = "edpktywJsAeturPxoFkDEerF6bi7N41ZnQyMrmNLQ3GZx2w6nn8eCZ";
    let priv_key = "edsk37Qf3bj5actYQj38hNnu5WtbYVw3Td7dxWQnV9XhrYeBYDuSty";

    Ok((
        PublicKey::from_base58check(pub_key).unwrap(),
        PrivateKey::from_base58check(priv_key).unwrap(),
    ))
}

impl Transfer {
    // TODO: fix transfer not working to new account
    pub fn execute(self) {
        let Transfer {
            // TODO: use verbose to print additional info
            verbose: _,
            endpoint,
            use_trezor,
            to,
            from: raw_from,
            amount: raw_amount,
            fee: raw_fee,
        } = self;

        let mut device = find_trezor_device();
        // device.set_debug_mode();
        let mut trezor = device.connect().unwrap();
        trezor.init_device().unwrap();

        let from = {
            let from = if use_trezor {
                // TODO: get address
                "tz1cQbQUb1EcrEwCgTPPGdoWmixFzRArozYW".to_string()
            } else {
                raw_from
            };

            match PublicKeyHash::from_base58check(&from) {
                Ok(pkh) => pkh,
                Err(err) => {
                    exit_with_error(format!(
                        "invalid {} public key hash: {}",
                        style("--from").bold(),
                        style(from).magenta(),
                    ));
                }
            }
        };

        let to = match PublicKeyHash::from_base58check(&to) {
            Ok(pkh) => pkh,
            Err(_) => {
                exit_with_error(format!(
                    "invalid {} public key hash: {}",
                    style("--from").bold(),
                    style(to).magenta(),
                ));
            }
        };

        let amount = match parse_float_amount(&raw_amount) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid amount: {}",
                    style(&raw_amount).bold()
                ));
            }
        };

        let fee = match parse_float_amount(&raw_fee) {
            Ok(amount) => amount,
            Err(_) => {
                exit_with_error(format!(
                    "invalid fee: {}",
                    style(&raw_amount).bold()
                ));
            }
        };

        // TODO: accept this as generic parameter instead
        let client = lib::http_api::HttpApi::new(endpoint);

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[1/4]").bold().dim())
            .with_text("fetching necessary data from the node")
            .start();

        let protocol_info = client.get_protocol_info().unwrap();
        let counter = client.get_counter_for_key(&from).unwrap() + 1;
        let constants = client.get_constants().unwrap();
        let head_block_hash = client.get_head_block_hash().unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[1/4]").bold().green(),
            emojies::TICK,
            "fetched necessary data from the node",
        );

        let tx = NewTransactionOperationBuilder::new()
            .source(from.clone())
            .destination(to.clone())
            .amount(amount.to_string())
            .fee(fee.to_string())
            .counter(counter.to_string())
            .gas_limit(50000.to_string())
            .storage_limit(constants.hard_storage_limit_per_operation.to_string())
            .build()
            .unwrap();


        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[2/4]").bold().dim())
            .with_text("forging the operation and signing")
            .start();

        let operation_group = NewOperationGroup::new(head_block_hash.clone())
            .with_transaction(tx);

        let forged_operation = client.forge_operations(&head_block_hash, &operation_group).unwrap();

        let sig_info = {
            if !use_trezor {
                let local_signer = {
                    let (pub_key, priv_key) = match get_keys_by_pkh(&from) {
                        Ok(keys) => keys,
                        Err(_) => {
                            exit_with_error(format!(
                                    "no local wallet with public key hash: {}",
                                    style(from.to_base58check()).bold()
                                    ));
                        }
                    };
                    LocalSigner::new(pub_key, priv_key)
                };

                local_signer.sign_operation(forged_operation.clone()).unwrap()
            } else {
                let mut tx: TezosSignTx = operation_group.into();
                tx.set_address_n(parse_derivation_path("m/44'/1729'/0'"));
                dbg!(
                    trezor_execute(
                        trezor.sign_tx(tx)
                    )
                );
                exit_with_error("");
            }
        };
        let signature = sig_info.signature.clone();
        let operation_with_signature = sig_info.operation_with_signature.clone();
        let operation_hash = sig_info.operation_hash.clone();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[2/4]").bold().green(),
            emojies::TICK,
            "operation forged and signed",
        );

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[3/4]").bold().dim())
            .with_text("applying and injecting the operation")
            .start();

        dbg!(client.preapply_operations(
            &protocol_info.next_protocol_hash,
            &head_block_hash,
            &signature,
            &operation_group,
        ).unwrap());

        client.inject_operations(&operation_with_signature).unwrap();

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[3/4]").bold().green(),
            emojies::TICK,
            "applied and injected the operation",
        );

        let spinner = SpinnerBuilder::new()
            .with_prefix(style("[4/4]").bold().dim())
            .with_text("waiting for confirmation")
            .start();

        for _ in 0..10 {
            thread::sleep(Duration::from_secs(2));

            let status = client.get_pending_operation_status(&operation_hash).unwrap();
            match status {
                PendingOperationStatus::Refused => {
                    exit_with_error("transaction refused");
                }
                PendingOperationStatus::Applied => {
                }
                PendingOperationStatus::Finished => {
                    break;
                }
            }
        }

        spinner.finish();
        eprintln!(
            "{} {} {}",
            style("[4/4]").bold().green(),
            emojies::TICK,
            "operation confirmed",
        );
        eprintln!();

        eprintln!(
            "  {}View operation at: {}/{}",
            emojies::FINGER_POINTER_RIGHT,
            style("https://delphinet.tezblock.io/transaction").cyan(),
            style(&operation_hash).cyan(),
        );

        if !console::user_attended() {
            println!("{}", &operation_hash);
        }
    }
}
