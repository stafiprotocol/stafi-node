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

#[cfg(test)]
mod tests {
    use super::super::message_utils;

    #[test]
    fn test_write_int() {
        let mut result = message_utils::write_int(7);
        assert_eq!(result, "07");

        result = message_utils::write_int(32);
        assert_eq!(result, "20");

        result = message_utils::write_int(4096);
        assert_eq!(result, "8020");

        result = message_utils::write_int(0);
        assert_eq!(result, "00");

        result = message_utils::write_signed_int(0);
        assert_eq!(result, "00");

        result = message_utils::write_signed_int(-64);
        assert_eq!(result, "c001");

        result = message_utils::write_signed_int(-120053);
        assert_eq!(result, "f5d30e");

        result = message_utils::write_signed_int(30268635200);
        assert_eq!(result, "80e1b5c2e101");

        result = message_utils::write_signed_int(610913435200);
        assert_eq!(result, "80f9b9d4c723");
    }

    #[test]
    fn test_write_address() {
        let mut result = message_utils::write_address("tz1Y68Da76MHixYhJhyU36bVh7a8C9UmtvrR").unwrap_or_default();
        assert_eq!(result, "00008890efbd6ca6bbd7771c116111a2eec4169e0ed8");

        result = message_utils::write_address("tz2LBtbMMvvguWQupgEmtfjtXy77cHgdr5TE").unwrap_or_default();
        assert_eq!(result, "0001823dd85cdf26e43689568436e43c20cc7c89dcb4");

        result = message_utils::write_address("tz3e75hU4EhDU3ukyJueh5v6UvEHzGwkg3yC").unwrap_or_default();
        assert_eq!(result, "0002c2fe98642abd0b7dd4bc0fc42e0a5f7c87ba56fc");

        result = message_utils::write_address("KT1NrjjM791v7cyo6VGy7rrzB3Dg3p1mQki3").unwrap_or_default();
        assert_eq!(result, "019c96e27f418b5db7c301147b3e941b41bd224fe400");
    }
}