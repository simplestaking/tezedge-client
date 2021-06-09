use std::thread;
use std::fmt::{self, Display};
use std::time::Duration;
use std::process::{Command, Stdio};
use std::error::Error;
use std::sync::Mutex;
use structopt::StructOpt;
use console::{style, Term};

use lib::{Address, ImplicitAddress, PublicKey, ToBase58Check};
use lib::api::*;
use lib::http_api::HttpApi;
use lib::explorer_api::TzStats;
use cli_spinner::SpinnerBuilder;

#[derive(thiserror::Error, Debug)]
enum BuildError {
    #[error("cargo build failed! IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("cargo build failed! Raw Output:\n{0}")]
    Output(String),
}

#[derive(thiserror::Error, Debug)]
struct CommandError {
    command: String,
    output: String,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Cli command failed!\nCommand: {}\n\n Raw Output:\n\n{}",
            self.command,
            self.output,
        )
    }
}

/// Test cli functionalities.
///
/// Pass public and private keys of the account, which will be the
/// source of funds. Needs to have tezos in balance!
#[derive(StructOpt)]
pub struct TestAll {
    #[structopt(short = "E", long)]
    pub endpoint: String,

    #[structopt(long = "public-key")]
    pub public_key: String,

    #[structopt(long = "private-key")]
    pub private_key: String,

    // #[structopt(long = "trezor")]
    // pub use_trezor: bool,

    // #[structopt(long = "ledger")]
    // pub use_ledger: bool,

    /// Use --release mode when building: `cargo build --release`.
    #[structopt(long)]
    pub release: bool,

    #[structopt(skip)]
    version: Mutex<Option<VersionInfo>>,
}

impl TestAll {
    fn api(&self) -> HttpApi {
        HttpApi::new(&self.endpoint)
    }

    fn explorer(&self) -> TzStats {
        let store = &mut *self.version.lock().unwrap();
        let version = if let Some(version) = store.as_ref() {
            version.clone()
        } else {
            let version = self.api().get_version_info().unwrap();
            *store = Some(version.clone());
            version
        };

        TzStats::new(version.get_network()).unwrap()
    }

    fn cli_command(&self) -> Command {
        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| String::from("target"));
        let cli_path = format!(
            "{}/{}/tezedge-client",
            target_dir,
            if self.release { "release" } else { "debug" },
        );

        let mut command = Command::new(cli_path);
        // enable colors
        command.env("CLICOLOR", "1");
        command.env("CLICOLOR_FORCE", "1");

        command
    }

    fn spinner_for_build(&self) -> SpinnerBuilder {
        let mut spinner = SpinnerBuilder::new()
            .with_text("cargo build");

        if self.release {
            spinner = spinner.with_text("cargo build --release")
        }

        spinner
    }

    fn build(&self) -> Result<(), BuildError> {
        let mut command = Command::new("cargo");
        command.arg("build");
        // enable colors
        command.arg("--color").arg("always");

        if self.release {
            command.arg("--release");
        }

        let output = command.output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(BuildError::Output(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    fn get_address_command(&self, key_path: &str, device_flag: &str) -> Command {
        let mut command = self.cli_command();
        command
            .arg("address").arg("get")
            .arg(device_flag)
            .arg("--path").arg(key_path)
            .stderr(Stdio::inherit());
        command
    }

    fn get_address(&self, key_path: &str, device_flag: &str) -> Result<Address, Box<dyn Error>> {
        let mut command = self.get_address_command(key_path, device_flag);
        let command_str = format!("{:?}", &command);
        let output = command.output()?;

        if output.status.success() {
            let addr_str = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            Ok(Address::from_base58check(&addr_str)?)
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            }.into())
        }
    }

    fn hw_originate_command(
        &self,
        device: &str,
        key_path: &str,
        balance: &str,
        fee: &str,
    ) -> Command {
        let mut command = self.cli_command();
        command
            .arg("originate")
            .arg("--no-prompt")
            .arg("--endpoint").arg(&self.endpoint)
            .arg(format!("--{}", device))
            .arg("--key-path").arg(key_path)
            .arg("--balance").arg(balance)
            .arg("--fee").arg(fee);
        command
    }

    fn hw_originate(
        &self,
        device: &str,
        key_path: &str,
        balance: &str,
        fee: &str,
    ) -> Result<Address, Box<dyn Error>> {
        let mut command = self.hw_originate_command(device, key_path, balance, fee);
        let command_str = format!("{:?}", &command);
        let output = command.output()?;

        if !output.status.success() {
            return Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            }.into());
        }

        let op_hash = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        let mut op_result = self.explorer().get_operation(&op_hash);
        for _ in 0..10 {
            match op_result {
                Ok(op) => {
                    let contract_address = op[0].receiver.clone();
                    return Ok(contract_address);
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(1000));
            op_result = self.explorer().get_operation(&op_hash);
        }

        Err(op_result.err().unwrap().into())
    }

    fn transfer_local_command(
        &self,
        from: &Address,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Command
    {
        let mut command = self.cli_command();
        command
            .arg("transfer-local")
            .arg("--no-prompt")
            .arg("--endpoint").arg(&self.endpoint)
            .arg("--private-key").arg(&self.private_key)
            .arg("--public-key").arg(&self.public_key)
            .arg("--from").arg(from.to_base58check())
            .arg("--to").arg(to.to_base58check())
            .arg("--amount").arg(amount);

        if let Some(fee) = fee {
            command.arg("--fee").arg(fee);
        }

        command
    }

    fn transfer_local(
        &self,
        from: &Address,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        let mut command = self.transfer_local_command(from, to, amount, fee.clone());
        let command_str = format!("{:?}", &command);

        let output = command.output().expect("failed to get output of transfer-local command!");

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn hw_transfer_command(
        &self,
        device_flag: &str,
        key_path: &str,
        from: Option<&Address>,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Command
    {
        let mut command = self.cli_command();
        command
            .arg("transfer")
            .arg("--no-prompt")
            .arg(device_flag)
            .arg("--endpoint").arg(&self.endpoint)
            .arg("--to").arg(to.to_base58check())
            .arg("--amount").arg(amount);

        if let Some(from) = from {
            command
                .arg("--from").arg(from.to_base58check())
                .arg("--key-path").arg(key_path);
        } else {
            command.arg("--from").arg(key_path);
        }

        if let Some(fee) = fee {
            command.arg("--fee").arg(fee);
        }

        command
    }

    fn hw_transfer(
        &self,
        device_flag: &str,
        key_path: &str,
        from: Option<&Address>,
        to: &Address,
        amount: &str,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        let mut command = self.hw_transfer_command(
            device_flag, key_path, from, to, amount, fee.clone(),
        );
        let command_str = format!("{:?}", &command);

        let output = command.output().expect("failed to get output of transfer command!");

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn hw_delegate_command(
        &self,
        device_flag: &str,
        key_path: &str,
        from: Option<&Address>,
        to: Option<&ImplicitAddress>,
        fee: Option<&str>,
    ) -> Command
    {
        let mut command = self.cli_command();
        command
            .arg("delegate")
            .arg("--no-prompt")
            .arg(device_flag)
            .arg("--endpoint").arg(&self.endpoint);

        if let Some(from) = from {
            command
                .arg("--from").arg(from.to_base58check())
                .arg("--key-path").arg(key_path);
        } else {
            command.arg("--from").arg(key_path);
        }

        if let Some(to) = to {
            command.arg("--to").arg(to.to_base58check());
        } else {
            command.arg("--cancel");
        }

        if let Some(fee) = fee {
            command.arg("--fee").arg(fee);
        }

        command
    }

    fn hw_delegate(
        &self,
        device_flag: &str,
        key_path: &str,
        from: Option<&Address>,
        to: Option<&ImplicitAddress>,
        fee: Option<&str>,
    ) -> Result<String, CommandError>
    {
        let mut command = self.hw_delegate_command(
            device_flag, key_path, from, to, fee.clone(),
        );
        let command_str = format!("{:?}", &command);

        let output = command.output().expect("failed to get output of delegate command!");

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(CommandError {
                command: command_str,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    fn hw_find_unrevealed(
        &self,
        device: &str,
        prefix: &str,
        index: u32,
    ) -> Result<(u32, String, Address), Box<dyn Error>>
    {
        let rpc = self.api();
        let device_flag = format!("--{}", device);
        let mut i = index;

        loop {
            if i != index {
                let _ = Term::stderr().clear_last_lines(1);
            }
            let path = format!("{}/{}'", prefix, i);
            eprintln!(
                " -   searching for unrevealed {} account. Checking path: {}",
                style(device).bold(),
                style(&path).magenta(),
            );

            let address = self.get_address(&path, &device_flag)?;

            if rpc.get_manager_public_key(&address)?.is_none() {
                let _ = Term::stderr().clear_last_lines(1);
                eprintln!(
                    " {} {} unrevealed account ({}) found on path: {}",
                    style(emojies::TICK),
                    style(device).bold(),
                    style(address.to_base58check()).bold(),
                    style(&path).magenta(),
                );
                break Ok((i, path, address));
            }

            i += 1;
        }
    }

    /// Test transfer + reveal from hardware wallet.
    fn test_hw_transfer_reveal(
        &self,
        device: &str,
        key_path: &str,
        to: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("transfer + reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_transfer(&device_flag, key_path, None, to, "0.1", None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("transfer + reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test transfer without reveal from hardware wallet.
    fn test_hw_transfer(
        &self,
        device: &str,
        key_path: &str,
        to: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("transfer w/o reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_transfer(&device_flag, key_path, None, to, "0.1", None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("transfer w/o reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test transfer from hardware wallet originated contract.
    fn test_hw_originated_transfer(
        &self,
        device: &str,
        key_path: &str,
        from: &Address,
        to: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} {} account to {} account",
                style("transfer from originated").yellow(),
                style(device).bold(),
                style(to.type_str()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_transfer(&device_flag, key_path, Some(from), to, "0.1", None),
        )?;
        spinner.finish_succeed(format!(
            "{} {} account to {} account successful. Operation hash: {}",
            style("transfer from originated").green(),
            style(device).bold(),
            style(to.type_str()).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test delegate + reveal from hardware wallet.
    fn test_hw_delegate_reveal(
        &self,
        device: &str,
        key_path: &str,
        to: &ImplicitAddress,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("delegate + reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_delegate(&device_flag, key_path, None, Some(to), None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("delegate + reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test delegate without reveal from hardware wallet.
    fn test_hw_delegate(
        &self,
        device: &str,
        key_path: &str,
        to: &ImplicitAddress,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("delegate w/o reveal").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_delegate(&device_flag, key_path, None, Some(to), None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("delegate w/o reveal").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test delegate cancellation from hardware wallet.
    fn test_hw_delegate_cancel(
        &self,
        device: &str,
        key_path: &str,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} from {} account",
                style("delegate cancellation").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_delegate(&device_flag, key_path, None, None, None),
        )?;
        spinner.finish_succeed(format!(
            "{} from {} account successful. Operation hash: {}",
            style("delegate cancellation").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test delegation from hardware wallet originated contract.
    fn test_hw_originated_delegate(
        &self,
        device: &str,
        key_path: &str,
        from: &Address,
        to: &ImplicitAddress,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} {} account",
                style("delegation from originated").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_delegate(&device_flag, key_path, Some(from), Some(to), None),
        )?;
        spinner.finish_succeed(format!(
            "{} {} account successful. Operation hash: {}",
            style("delegation from originated").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    /// Test delegation cancellation from hardware wallet originated contract.
    fn test_hw_originated_delegate_cancel(
        &self,
        device: &str,
        key_path: &str,
        from: &Address,
    ) -> Result<(), Box<dyn Error>>
    {
        let device_flag = "--".to_string() + device;
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "testing {} {} account",
                style("delegate cancellation from originated").yellow(),
                style(device).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.hw_delegate(&device_flag, key_path, Some(from), None, None),
        )?;
        spinner.finish_succeed(format!(
            "{} {} account successful. Operation hash: {}",
            style("delegate cancellation from originated").green(),
            style(device).bold(),
            style(op_hash).cyan(),
        ));

        Ok(())
    }

    pub fn test(self) -> Result<(), Box<dyn Error>> {
        let mut spinner = self.spinner_for_build().start();
        spinner.fail_if(self.build())?;
        spinner.finish_succeed("build successful!");

        let key_path_prefix = "m/44'/1729'/0'".to_string();
        let public_key = PublicKey::from_base58check(&self.public_key)
            .map_err(|_| "invalid --public-key passed")?;
        let local_address: Address = public_key.hash().into();

        let (key_path_i, trezor_key_path, trezor_address)
            = self.hw_find_unrevealed("trezor", &key_path_prefix, 0)?;
        let (_, ledger_key_path, ledger_address)
            = self.hw_find_unrevealed("ledger", &key_path_prefix, key_path_i)?;

        // Transfer to trezor account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to trezor account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &trezor_address, "2", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to trezor account. Operation hash: {}",
            style(op_hash).cyan(),
        ));

        // Transfer to ledger account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to ledger account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &ledger_address, "2", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to ledger account. Operation hash: {}",
            style(op_hash).cyan(),
        ));
        eprintln!();

        // test transfer + reveal
        self.test_hw_transfer_reveal("trezor", &trezor_key_path, &ledger_address)?;
        self.test_hw_transfer_reveal("ledger", &ledger_key_path, &trezor_address)?;
        eprintln!();

        // test transfer with already revealed accounts
        self.test_hw_transfer("trezor", &trezor_key_path, &ledger_address)?;
        self.test_hw_transfer("ledger", &ledger_key_path, &trezor_address)?;
        eprintln!();

        let bakers = self.explorer().get_bakers()?;

        let (i, trezor_key_path, trezor_address)
            = self.hw_find_unrevealed("trezor", &key_path_prefix, key_path_i)?;
        let (_, ledger_key_path, ledger_address)
            = self.hw_find_unrevealed("ledger", &key_path_prefix, i)?;

        // Transfer to trezor account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to trezor account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &trezor_address, "5", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to trezor account. Operation hash: {}",
            style(op_hash).cyan(),
        ));

        // Transfer to ledger account.
        let mut spinner = SpinnerBuilder::new()
            .with_text(format!(
                "transfering funds to ledger account: {}",
                style(trezor_address.to_base58check()).bold(),
            ))
            .start();

        let op_hash = spinner.fail_if(
            self.transfer_local(&local_address, &ledger_address, "5", None),
        )?;
        spinner.finish_succeed(format!(
            "funds transfered to ledger account. Operation hash: {}",
            style(op_hash).cyan(),
        ));
        eprintln!();

        // test delegate + reveal
        self.test_hw_delegate_reveal("trezor", &trezor_key_path, &bakers[0].address)?;
        self.test_hw_delegate_reveal("ledger", &ledger_key_path, &bakers[0].address)?;
        eprintln!();

        // test delegate
        self.test_hw_delegate("trezor", &trezor_key_path, &bakers[1].address)?;
        self.test_hw_delegate("ledger", &ledger_key_path, &bakers[1].address)?;
        eprintln!();

        // test cancel
        self.test_hw_delegate_cancel("trezor", &trezor_key_path)?;
        self.test_hw_delegate_cancel("ledger", &ledger_key_path)?;
        eprintln!();

        let trezor_contract_address = self.hw_originate("trezor", &trezor_key_path, "2", "0.1")?;
        eprintln!(
            "originated {} contract: {}",
            style("trezor").bold(),
            style(trezor_contract_address.to_base58check()).bold(),
        );

        let ledger_contract_address = self.hw_originate("ledger", &ledger_key_path, "2", "0.1")?;
        eprintln!(
            "originated {} contract: {}",
            style("ledger").bold(),
            style(ledger_contract_address.to_base58check()).bold(),
        );
        eprintln!();

        // test transfer from originated address to implicit address
        self.test_hw_originated_transfer(
            "trezor",
            &trezor_key_path,
            &trezor_contract_address,
            &ledger_address,
        )?;
        self.test_hw_originated_transfer(
            "ledger",
            &ledger_key_path,
            &ledger_contract_address,
            &trezor_address,
        )?;
        eprintln!();

        // test transfer from originated address to originated address
        self.test_hw_originated_transfer(
            "trezor",
            &trezor_key_path,
            &trezor_contract_address,
            &ledger_contract_address,
        )?;
        self.test_hw_originated_transfer(
            "ledger",
            &ledger_key_path,
            &ledger_contract_address,
            &trezor_contract_address,
        )?;
        eprintln!();

        // test delegation from originated address
        self.test_hw_originated_delegate(
            "trezor",
            &trezor_key_path,
            &trezor_contract_address,
            &bakers[0].address,
        )?;
        self.test_hw_originated_delegate(
            "ledger",
            &ledger_key_path,
            &ledger_contract_address,
            &bakers[0].address,
        )?;
        eprintln!();

        // test delegation cancellation from originated address
        self.test_hw_originated_delegate_cancel(
            "trezor",
            &trezor_key_path,
            &trezor_contract_address,
        )?;
        self.test_hw_originated_delegate_cancel(
            "ledger",
            &ledger_key_path,
            &ledger_contract_address,
        )?;

        Ok(())
    }
}

fn main() {
    match TestAll::from_args().test() {
        Ok(_) => {}
        Err(_) => std::process::exit(1),
    }
}
