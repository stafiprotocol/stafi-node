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
extern crate regex;
extern crate alloc;

use regex::Regex;
use alloc::borrow::Cow::*;
use core::str;
use alloc::string;

// trait RegexReplacement {
//     fn replace_with_regex(&self, pattern: &str, replacement: &str) -> &str;
// }

// impl RegexReplacement for &str {
//     fn replace_with_regex(&self, pattern: &str, replacement: &str) -> &str {
//         let re = Regex::new(pattern).unwrap();

//         match re.replace_all(self, replacement) {
//             Borrowed(s) => s,
//             Owned(s) => &s,
//         }
//     }
// }

fn replace_with_regex(s: &str, pattern: &str, replacement: &str) -> String {
    let re = Regex::new(pattern).unwrap();

    match re.replace_all(s, replacement) {
        Borrowed(s) => s.to_string(),
        Owned(s) => s,
    }
}

/**
 * Micheline parser expects certain whitespace arrangement, this function will correct the input string accordingly.
 */
pub fn normalize_micheline_whitespace(fragment: &str) -> String {
    let mut result = fragment.to_string();
    result = replace_with_regex(&result, r"\n", " ");
    result = replace_with_regex(&result, r#" +"#, r#" "#);
    result = result.replace(r#"[{"#, r#"[ {"#);
    result = result.replace(r#"}]"#, r#"} ]"#);
    result = result.replace(r#"},{"#, r#"}, {"#);
    result = result.replace(r#"]}"#, r#"] }"#);
    result = result.replace(r#"":""#, r#"": ""#);
    result = result.replace(r#"":["#, r#"": ["#);
    result = result.replace(r#"{""#, r#"{ ""#);
    result = result.replace(r#""}"#, r#"" }"#);
    result = result.replace(r#",""#, r#", ""#);
    result = result.replace(r#"",""#, r#"", ""#);
    result = result.replace(r#"[["#, r#"[ ["#);
    result = result.replace(r#"]]"#, r#"] ]"#);
    result = result.replace(r#"[""#, r#"[ ""#);
    result = result.replace(r#""]"#, r#"" ]"#);
    result
}
#[cfg(test)]
#[test]
fn test_normalize() {
    let expected = r#"{ "prim": "NIL", "args": [ { "prim": "operation" }, { "prim": "operation" } ] , "annots": [ "@cba" ] }"#;
    let raw = r#"{      "prim":     "NIL", "args": [ { "prim": "operation" }, { "prim": "operation" } ]   , "annots":    [ "@cba" ] }"#;
    let result = normalize_micheline_whitespace(raw);
    assert_eq!(expected, result);
}
