// Copyright 2018 Stafi Protocol, Inc.
// This file is part of Stafi.

// Stafi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

extern crate base_custom;
extern crate hex;
use base_custom::BaseCustom;

pub fn encode(src: u64) -> String {
    let base_vec: Vec<String> = (0..128).map(|x| format!("{:02x}", x)).collect();
    let delim = " ";
    let salt = base_vec.join(delim);
    let base_music = BaseCustom::<String>::new(salt, Some(' '));

    let out_str = base_music.gen(src);
    let hex_vec = hex::decode(out_str.replace(" ", "")).unwrap();
    let mut new_hex: Vec<u8> = hex_vec
        .iter()
        .enumerate()
        .map(|(i, &v)| if i == 0 { v } else { v ^ 0x80 })
        .collect();
    new_hex.reverse();
    hex::encode(&new_hex)
}

#[cfg(test)]
mod tests {
    use super::super::base128;
    #[test]
    fn test_base128_encoding() {
        let test_num = 1143410;
        let test_result = "f2e445";
        let encoded = base128::encode(test_num);
        assert_eq!(encoded, test_result);
    }
}
