// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use crate::{chain_spec, service, Cli, Subcommand};
use node_executor::Executor;
use node_runtime::{Block, RuntimeApi};
use sc_cli::{Result, SubstrateCli, RuntimeVersion, Role, ChainSpec};
use sc_service::PartialComponents;
use crate::service::new_partial;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Stafi Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/stafiprotocol/stafi-node/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()),
			"local" => Box::new(chain_spec::local_testnet_config()),
			"public-test" => Box::new(chain_spec::stafi_public_testnet_config()),
			"testnet" => Box::new(chain_spec::stafi_testnet_config()?),
			"incentive-test" => Box::new(chain_spec::stafi_incentive_testnet_config()),
			"sitara" => Box::new(chain_spec::stafi_sitara_testnet_config()?),
            "" | "mainnet" => Box::new(chain_spec::stafi_mainnet_config()?),
			path => Box::new(chain_spec::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&node_runtime::VERSION
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| match config.role {
				Role::Light => service::new_light(config),
				_ => service::new_full(config),
			})
		}
		Some(Subcommand::Inspect(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| cmd.run::<Block, RuntimeApi, Executor>(config))
		}
		Some(Subcommand::Benchmark(cmd)) => {
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;

				runner.sync_run(|config| cmd.run::<Block, Executor>(config))
			} else {
				println!("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`.");
				Ok(())
			}
		}
		Some(Subcommand::Base(subcommand)) => {
			let runner = cli.create_runner(subcommand)?;
			runner.run_subcommand(subcommand, |config| {
				let PartialComponents { client, backend, task_manager, import_queue, ..}
					= new_partial(&config)?;
				Ok((client, backend, import_queue, task_manager))
			})
		}
	}
}
