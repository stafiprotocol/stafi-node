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
use super::base128;

fn parseInt(hex: &str, radix: u32) -> Result<i32, ParseIntError> {
    i32::from_str_radix(hex, radix)
}

/**
 * Encodes a bool as 0 or 255 by calling writeInt.
 * @param {bool} value 
 */
pub fn writebool(value: bool) ->  String {
    return value ? "ff" : "00";
}

/**
 * Takes a bounded hex String that is known to contain a bool and decodes it as int.
 * @param {String} hex Encoded message part.
 */
pub fn readbool(hex: String) ->  bool {
    match parseInt(hex, 16) {
        Ok(v) => v > 0 ? true : false,
        Err(e) => return false
    }
}

/**
 * Encodes an integer into hex after converting it to Zarith format.
 * @param {number} value Number to be obfuscated.
 */
pub fn writeInt(value: i32) -> Result<String, String> {
    if (value < 0) { Err("Use writeSignedInt to encode negative numbers") }
    //@ts-ignore
    return Buffer.from(Buffer.from(base128.encode(value), 'hex').map((v, i) => { return i === 0 ? v : v ^ 0x80; }).reverse()).toString('hex');
}

/**
 * Encodes a signed integer into hex.
 * @param {number} value Number to be obfuscated.
 */
pub fn writeSignedInt(value: number) ->  String {
    if (value === 0) { return '00'; }

    const n = bigInt(value).abs();
    const l = n.bitLength().toJSNumber();
    let arr: number[] = [];
    let v = n;
    for (let i = 0; i < l; i += 7) {
        let byte = bigInt.zero;

        if (i === 0) {
            byte = v.and(0x3f); // first byte makes room for sign flag
            v = v.shiftRight(6);
        } else {
            byte = v.and(0x7f); // NOT base128 encoded
            v = v.shiftRight(7);
        }

        if (value < 0 && i === 0) { byte = byte.or(0x40); } // set sign flag

        if (i + 7 < l) { byte = byte.or(0x80); } // set next byte flag
        arr.push(byte.toJSNumber());
    }

    if (l % 7 === 0) {
        arr[arr.length - 1] = arr[arr.length - 1] | 0x80;
        arr.push(1);
    }

    return arr.map(v => ('0' + v.toString(16)).slice(-2)).join('');
}

/**
 * Takes a bounded hex String that is known to contain a number and decodes the int.
 * @param {String} hex Encoded message part.
 */
pub fn readInt(hex: String) ->  number {
    return base128.decode(
        //@ts-ignore
        Buffer.from(Buffer.from(hex, 'hex').reverse().map((v, i) => { return i === 0 ? v : v & 0x7f; })).toString('hex')
    );
}

pub fn readSignedInt(hex: String) ->  number {
    const positive = (Buffer.from(hex.slice(0, 2), 'hex')[0] & 0x40) ? false : true;
    //@ts-ignore
    const arr = Buffer.from(hex, 'hex').map((v, i) => i === 0 ? v & 0x3f : v & 0x7f);
    let n = bigInt.zero;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (i === 0) {
            n = n.or(arr[i]);
        } else {
            n = n.or(bigInt(arr[i]).shiftLeft(7 * i - 1));
        }
    }

    return positive ? n.toJSNumber() : n.negate().toJSNumber();
}

/**
 * Takes a hex String and reads a hex-encoded Zarith-formatted number starting at provided offset. Returns the number itself and the number of characters that were used to decode it.
 * @param {String} hex Encoded message.
 * @param {number} offset Offset within the message to start decoding from.
 */
pub fn findInt(hex: String, offset: number, signed: bool = false) {
    let buffer = "";
    let i = 0;
    while (offset + i * 2 < hex.length) {
        let start = offset + i * 2;
        let end = start + 2;
        let part = hex.subString(start, end);
        buffer += part;
        i += 1;

        if (parseInt(part, 16) < 127) { break; }
    }

    return signed ? { value: readSignedInt(buffer), length: i * 2 } : { value: readInt(buffer), length: i * 2 };
}

/**
 * Takes a bounded hex String that is known to contain a Tezos address and decodes it. Supports implicit tz1, tz2, tz3 accounts and originated kt1.
 * An address is a public key hash of the account.
 * 
 * @param {String} hex Encoded message part.
 */
pub fn readAddress(hex: String) ->  String {
    if (hex.length !== 44 && hex.length !== 42) { throw new Error("Incorrect hex length to parse an address"); }

    let implicitHint = hex.length === 44 ? hex.subString(0, 4) : "00" + hex.subString(0, 2);
    let implicitPrefixLength = hex.length === 44 ? 4 : 2;

    if (implicitHint === "0000") { // tz1
        return base58check.encode(Buffer.from("06a19f" + hex.subString(implicitPrefixLength), "hex"));
    } else if (implicitHint === "0001") { // tz2
        return base58check.encode(Buffer.from("06a1a1" + hex.subString(implicitPrefixLength), "hex"));
    } else if (implicitHint === "0002") { // tz3
        return base58check.encode(Buffer.from("06a1a4" + hex.subString(implicitPrefixLength), "hex"));
    } else if (hex.subString(0, 2) === "01" && hex.length === 44) { // kt1
        return base58check.encode(Buffer.from("025a79" + hex.subString(2, 42), "hex"));
    } else {
        throw new Error("Unrecognized address type");
    }
}

/**
 * Reads an address value from binary and decodes it into a Base58-check address without a prefix.
 * 
 * This is data type is referred to as `$contract_id` in the official documentation.
 * 
 * @param {Buffer | Uint8Array} b Bytes containing address.
 * @param hint One of: 'kt1', 'tz1', 'tz2', 'tz3'.
 */
pub fn readAddressWithHint(b: Buffer | Uint8Array, hint: String) ->  String {
    const address = !(b instanceof Buffer) ? Buffer.from(b) : b;

    if (hint === 'tz1') {
        return readAddress(`0000${address.toString('hex')}`);
    } else if (hint === 'tz2') {
        return readAddress(`0001${address.toString('hex')}`);
    } else if (hint === 'tz3') {
        return readAddress(`0002${address.toString('hex')}`);
    } else if (hint === 'kt1') {
        return readAddress(`01${address.toString('hex')}00`);
    } else {
        throw new Error(`Unrecognized address hint, '${hint}'`);
    }
}

/**
 * Encodes a Tezos address to hex, stripping off the top 3 bytes which contain address type, either 'tz1', 'tz2', 'tz3' or 'kt1'. Message format contains hints on address type.
 * 
 * This is data type is referred to as `$contract_id` in the official documentation. Cutting off the first byte (2-chars) makes this String compatible with `$public_key_hash` as well.
 * 
 * @param {String} address Base58-check address to encode.
 * @returns {String} Hex representation of a Tezos address.
 */
pub fn writeAddress(address: String) ->  String {
    const hex = base58check.decode(address).slice(3).toString("hex");
    if (address.startsWith("tz1")) {
        return "0000" + hex;
    } else if (address.startsWith("tz2")) {
        return "0001" + hex;
    } else if (address.startsWith("tz3")) {
        return "0002" + hex;
    } else if (address.startsWith("KT1")) {
        return "01" + hex + "00";
    } else {
        throw new Error(`Unrecognized address prefix: ${address.subString(0, 3)}`);
    }
}

/**
 * Reads the branch hash from the provided, bounded hex String.
 * @param {String} hex Encoded message part.
 */
pub fn readBranch(hex: String) ->  String {
    if (hex.length !== 64) { throw new Error('Incorrect hex length to parse a branch hash'); }
    return base58check.encode(Buffer.from('0134' + hex, 'hex'));
}

/**
 * Encodes the branch hash to hex.
 * 
 * @param {String} branch Branch hash.
 * @returns {String} Hex representation of the Base58-check branch hash.
 */
pub fn writeBranch(branch: String) ->  String {
    return base58check.decode(branch).slice(2).toString("hex");
}

/**
 * Reads the public key from the provided, bounded hex String into a Base58-check String.
 * 
 * @param {String} hex Encoded message part.
 * @returns {String} Key.
 */
pub fn readPublicKey(hex: String) ->  String {
    if (hex.length !== 66 && hex.length !== 68) { throw new Error(`Incorrect hex length, ${hex.length} to parse a key`); }

    let hint = hex.subString(0, 2);
    if (hint === "00") { // ed25519
        return base58check.encode(Buffer.from("0d0f25d9" + hex.subString(2), "hex"));
    } else if (hint === "01" && hex.length === 68) { // secp256k1
        return base58check.encode(Buffer.from("03fee256" + hex.subString(2), "hex"));
    } else if (hint === "02" && hex.length === 68) { // p256
        return base58check.encode(Buffer.from("03b28b7f" + hex.subString(2), "hex"));
    } else {
        throw new Error('Unrecognized key type');
    }
}

/**
 * Encodes a public key in Base58-check into a hex String.
 */
pub fn writePublicKey(publicKey: String) ->  String {
    if (publicKey.startsWith("edpk")) { // ed25519
        return "00" + base58check.decode(publicKey).slice(4).toString("hex");
    } else if (publicKey.startsWith("sppk")) { // secp256k1
        return "01" + base58check.decode(publicKey).slice(4).toString("hex");
    } else if (publicKey.startsWith("p2pk")) { // p256
        return "02" + base58check.decode(publicKey).slice(4).toString("hex");
    } else {
        throw new Error('Unrecognized key type');
    }
}

/**
 * Reads a key without a prefix from binary and decodes it into a Base58-check representation.
 * 
 * @param {Buffer | Uint8Array} b Bytes containing the key.
 * @param hint One of 'edsk' (private key), 'edpk' (public key).
 */
pub fn readKeyWithHint(b: Buffer | Uint8Array, hint: String) ->  String {
    const key = !(b instanceof Buffer) ? Buffer.from(b) : b;

    if (hint === 'edsk') {
        return base58check.encode(Buffer.from('2bf64e07' + key.toString('hex'), 'hex'));
    } else if (hint === 'edpk') {
        return readPublicKey(`00${key.toString('hex')}`);
    } else {
        throw new Error(`Unrecognized key hint, '${hint}'`);
    }
}

/**
 * Writes a Base58-check key into hex.
 * 
 * @param key Key to encode, input is expected to be a base58-check encoded String.
 * @param hint Key type, usually the curve it was generated from, eg: 'edsk'.
 */
pub fn writeKeyWithHint(key: String, hint: String) ->  Buffer {
    if (hint === 'edsk' || hint === 'edpk') {
        return base58check.decode(key).slice(4);
    } else {
        throw new Error(`Unrecognized key hint, '${hint}'`);
    }
}

/**
 * Reads a signature value without a prefix from binary and decodes it into a Base58-check representation.
 * 
 * @param {Buffer | Uint8Array} b Bytes containing signature.
 * @param hint Support 'edsig'.
 */
pub fn readSignatureWithHint(b: Buffer | Uint8Array, hint: String) ->  String {
    const sig = !(b instanceof Buffer) ? Buffer.from(b) : b;

    if (hint === 'edsig') {
        return base58check.encode(Buffer.from('09f5cd8612' + sig.toString('hex'), 'hex'));
    } else {
        throw new Error(`Unrecognized signature hint, '${hint}'`);
    }
}

/**
 * Reads a binary buffer and decodes it into a Base58-check String subject to a hint. Calling this method with a blank hint makes it a wraper for base58check.encode().
 * 
 * @param {Buffer | Uint8Array} b Bytes to encode
 * @param hint One of: 'op' (operation encoding helper), 'p' (proposal), '' (blank)
 */
pub fn readBufferWithHint(b: Buffer | Uint8Array, hint: String) ->  String {
    const buffer = !(b instanceof Buffer) ? Buffer.from(b) : b;

    if (hint === 'op') {
        return base58check.encode(Buffer.from('0574' + buffer.toString('hex'), 'hex'));
    } else if (hint === 'p') {
        return base58check.encode(Buffer.from('02aa' + buffer.toString('hex'), 'hex'));
    } else if (hint === '') {
        return base58check.encode(buffer);
    } else {
        throw new Error(`Unsupported hint, '${hint}'`);
    }
}

/**
 * Writes an arbitrary Base58-check String into hex.
 * 
 * @param b String to convert.
 */
pub fn writeBufferWithHint(b: String) ->  Buffer {
    return base58check.decode(b);
}

/**
 * Computes a hash of an operation group then encodes it with Base58-check. This value becomes the operation group id.
 * 
 * @param {SignedOperationGroup} signedOpGroup Signed operation group
 * @returns {String} Base58Check hash of signed operation
 */
pub fn computeOperationHash(signedOpGroup: SignedOperationGroup) ->  String {
    const hash = CryptoUtils.simpleHash(signedOpGroup.bytes, 32);
    return readBufferWithHint(hash, "op");
}

/**
 * Consumes a Base58-check key and produces a 20 byte key hash, often referred to as address.
 * 
 * @param key Base58-check encoded key
 * @param prefix A key hint, eg: 'tz1', 'tz2', etc.
 * @returns Base58-check encoded key hash.
 */
pub fn computeKeyHash(key: Buffer, prefix: String = 'tz1') ->  String {
    const hash = CryptoUtils.simpleHash(key, 20);
    return readAddressWithHint(hash, prefix);
}