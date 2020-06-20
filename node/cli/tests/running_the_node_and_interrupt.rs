// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use assert_cmd::cargo::cargo_bin;
use std::{convert::TryInto, process::Command, thread, time::Duration};
use tempfile::tempdir;

pub mod common;

#[test]
#[cfg(unix)]
fn running_the_node_works_and_can_be_interrupted() {
	use nix::sys::signal::{kill, Signal::{self, SIGINT, SIGTERM}};
	use nix::unistd::Pid;

	fn run_command_and_kill(signal: Signal) {
		let base_path = tempdir().expect("could not create a temp dir");
		let mut cmd = Command::new(cargo_bin("stafi"))
			.args(&["--dev", "-d"])
			.arg(base_path.path())
			.spawn()
			.unwrap();

		thread::sleep(Duration::from_secs(20));
		assert!(cmd.try_wait().unwrap().is_none(), "the process should still be running");
		kill(Pid::from_raw(cmd.id().try_into().unwrap()), signal).unwrap();
		assert_eq!(
			common::wait_for(&mut cmd, 30).map(|x| x.success()),
			Some(true),
			"the process must exit gracefully after signal {}",
			signal,
		);
	}

	run_command_and_kill(SIGINT);
	run_command_and_kill(SIGTERM);
}
