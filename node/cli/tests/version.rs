// Copyright 2019-2020 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

use assert_cmd::cargo::cargo_bin;
use platforms::*;
use regex::Regex;
use std::process::Command;

fn expected_regex() -> Regex {
	Regex::new(r"^stafi (\d+\.\d+\.\d+(?:-.+?)?)-([a-f\d]+|unknown)-(.+?)-(.+?)(?:-(.+))?$").unwrap()
}

#[test]
fn version_is_full() {
	let expected = expected_regex();
	let output = Command::new(cargo_bin("stafi"))
		.args(&["--version"])
		.output()
		.unwrap();

	assert!(
		output.status.success(),
		"command returned with non-success exit code"
	);

	let output = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	let captures = expected
		.captures(output.as_str())
		.expect("could not parse version in output");

	assert_eq!(&captures[1], env!("CARGO_PKG_VERSION"));
	assert_eq!(&captures[3], TARGET_ARCH.as_str());
	assert_eq!(&captures[4], TARGET_OS.as_str());
	assert_eq!(
		captures.get(5).map(|x| x.as_str()),
		TARGET_ENV.map(|x| x.as_str())
	);
}

#[test]
fn test_regex_matches_properly() {
	let expected = expected_regex();

	let captures = expected
		.captures("stafi 2.0.0-da487d19d-x86_64-linux-gnu")
		.unwrap();
	assert_eq!(&captures[1], "2.0.0");
	assert_eq!(&captures[2], "da487d19d");
	assert_eq!(&captures[3], "x86_64");
	assert_eq!(&captures[4], "linux");
	assert_eq!(captures.get(5).map(|x| x.as_str()), Some("gnu"));

	let captures = expected
		.captures("stafi 2.0.0-alpha.5-da487d19d-x86_64-linux-gnu")
		.unwrap();
	assert_eq!(&captures[1], "2.0.0-alpha.5");
	assert_eq!(&captures[2], "da487d19d");
	assert_eq!(&captures[3], "x86_64");
	assert_eq!(&captures[4], "linux");
	assert_eq!(captures.get(5).map(|x| x.as_str()), Some("gnu"));

	let captures = expected
		.captures("stafi 2.0.0-alpha.5-da487d19d-x86_64-linux")
		.unwrap();
	assert_eq!(&captures[1], "2.0.0-alpha.5");
	assert_eq!(&captures[2], "da487d19d");
	assert_eq!(&captures[3], "x86_64");
	assert_eq!(&captures[4], "linux");
	assert_eq!(captures.get(5).map(|x| x.as_str()), None);
}
