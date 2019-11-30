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

extern crate hex;

use core::str;

pub struct MultisigTransaction {
    counter: u64, // counter, used to prevent replay attacks
    amount: u64, // (mutez) amount to transfer
    dest: String, // destination to transfer to
    sig: Option<Vec<String>>, // sig list
}

pub struct Transaction {
    source: String, // pkh
    fee: u64, // 150000
    counter: u64, // account counter
    gas_limit: u64, // 144382
    storage_limit: u64, // 5392
    amount: u64, // 0
    destination: String, // contract address
    parameters: Option<MultisigTransaction>,
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

pub use crate::message_utils::*;

pub fn encode_transaction(branch: &str, transaction: &Transaction) -> String {
    let mut trans_hex = "".to_string();
    trans_hex += &write_branch(&branch);
    trans_hex += &write_transacion(&transaction);
    trans_hex
}

pub fn write_transacion(transaction: &Transaction) -> String {
    let mut trans_hex = "".to_string();
    
    trans_hex += &write_int(
        OPERATION_TYPES
            .iter()
            .position(|&r| r == "transaction")
            .unwrap() as u64,
    );

    trans_hex += &write_address(&transaction.source).unwrap_or_default()[2..];
    trans_hex += &write_int(transaction.fee);
    trans_hex += &write_int(transaction.counter);
    trans_hex += &write_int(transaction.gas_limit);
    trans_hex += &write_int(transaction.storage_limit);
    trans_hex += &write_int(transaction.amount);
    trans_hex += &write_address(&transaction.destination).unwrap_or_default();
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
    // trans_hex += 'ff' + ('0000000' + (result.length / 2).toString(16)).slice(-8) + result; // prefix byte length
    if let Some(params) = &transaction.parameters {
        trans_hex += "ff";
        // entrypoint == "default"
        trans_hex += "00";
        let result = write_multisig(&params);
        trans_hex += &(format!("{:08x}", result.len() / 2) + &result);
    } else {
        trans_hex += "00";
    }
    trans_hex
}

fn write_multisig(multisig: &MultisigTransaction) -> String {
    // 07070707{00/(signedInt, counter)01}05050707{00/(signedInt, amount)a301}{01{writeString, dest}0000000474657374}0200000000
    let counter = write_signed_int(multisig.counter as i64);
    let amount = write_signed_int(multisig.amount as i64);
    let dest = write_address(&multisig.dest).unwrap_or_default();
    format!("0707070700{}0505070700{}{}0200000000", counter, amount, dest)
}