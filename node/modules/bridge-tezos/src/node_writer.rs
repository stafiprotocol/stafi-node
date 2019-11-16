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

use core::str;

pub struct Transaction {
    source: String,
    fee: u64,
    counter: u64,
    gas_limit: u64,
    storage_limit: u64,
    amount: u64,
    destination: String,
    parameters: Option<String>,
}

static OPERATION_TYPES: &'static [&'static str] = &[
    "endorsement",
    "seedNonceRevelation",
    "doubleEndorsementEvidence",
    "doubleBakingEvidence",
    "accountActivation",
    "proposal",
    "ballot",
    "reveal",
    "transaction",
    "origination",
    "delegation",
];

use crate::message_utils::*;

pub fn write_transacion(transaction: Transaction) -> String {
    let mut hex = write_int(
        OPERATION_TYPES
            .iter()
            .position(|&r| r == "transaction")
            .unwrap() as u64,
    );

    hex += &write_address(&transaction.source).unwrap_or_default();
    hex += &write_int(transaction.fee);
    hex += &write_int(transaction.counter);
    hex += &write_int(transaction.gas_limit);
    hex += &write_int(transaction.storage_limit);
    hex += &write_int(transaction.amount);
    hex += &write_address(&transaction.destination).unwrap_or_default();
    //  * @param {number} counter - (nat) counter, used to prevent replay attacks
    //  * @param {number} amount - (mutez) amount to transfer
    //  * @param {string} dest - (contract unit) destination to transfer to
    //  * @param {string} sigs - (list (option signature)) signatures
    // const parameter = 'Pair (Pair '+ counter + ' (Left (Pair ' + amount + ' ' + dest + '))) ' + sigs;
    // TezosParameterFormat.Michelson
    // if (!!parameters && parameters.trim().length > 0) {
    //         if (parameterFormat === TezosTypes.TezosParameterFormat.Michelson) {
    //             const michelineParams = TezosLanguageUtil.translateMichelsonToMicheline(parameters);
    //             transaction.parameters = JSON.parse(michelineParams);
    //         } else if (parameterFormat === TezosTypes.TezosParameterFormat.Micheline) {
    //             transaction.parameters = JSON.parse(parameters);
    //         }
    //     }
    // const code = TezosLanguageUtil.normalizeMichelineWhiteSpace(JSON.stringify(transaction.parameters));
    // const result = TezosLanguageUtil.translateMichelineToHex(code);
    // hex += 'ff' + ('0000000' + (result.length / 2).toString(16)).slice(-8) + result; // prefix byte length
    if let Some(params) = transaction.parameters {
    } else {
        hex += "00";
    }
    hex
}
