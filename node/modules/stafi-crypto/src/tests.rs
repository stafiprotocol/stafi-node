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
    // Note this useful idiom: importing names from outer (for mod tests) scope.a
    use super::super::tez::sign::*;
    use hex_literal::*;

    #[test]
    fn test_tez_sign() {
        let data = hex!("7a922d798a40a2658a6bd744864f460e181343b3d6934585a3098b1fb5496cf8070000b80e26e0c5a4a1cb6f3b4df1c84570eef8ce9cf000ca8301000000bbdb70f173145a8b851a2863cd19598bea9c4ef783f4e0717b0338a8d811ea65080000b80e26e0c5a4a1cb6f3b4df1c84570eef8ce9cf000cb8301904e00c0843d00000387f1fb7d6831bf89b4a4d0953ce41cad88924900");
        let except_edsig = "edsigteQsDv3asrSke7Bf5p1uaqceSjni5LUhnDFBcqEBv8ZF1tdbQLwWAkxA61B5YMLqrFc6MQ8kBhfuYWch3gzQ5yfwgjWBbC";
        let except_sbytes = hex!("7a922d798a40a2658a6bd744864f460e181343b3d6934585a3098b1fb5496cf8070000b80e26e0c5a4a1cb6f3b4df1c84570eef8ce9cf000ca8301000000bbdb70f173145a8b851a2863cd19598bea9c4ef783f4e0717b0338a8d811ea65080000b80e26e0c5a4a1cb6f3b4df1c84570eef8ce9cf000cb8301904e00c0843d00000387f1fb7d6831bf89b4a4d0953ce41cad88924900327ce22ee35ac3bfdb1aea728aaf81c4467c4e75e68bf9e49751bb36b6399bc7b0c7efa6551731876654a195ec0b70f16bc3a12f44e5f898ab3d375262ac6904");

        let sk_with_prefix = "edskRiwkRKDGnXwxYgSNFx68vhLQ23RqVhYpVxjRmoUyKMjtEfeoREieDdGBk9meBPSMnYt5UnhxsgEyGd9EFm3mojgdVMkMBq";
        let sk = sk_with_prefix;
        let signature_data = sign(data.to_vec(), sk);
        assert_eq!(signature_data.edsig, except_edsig);
        assert_eq!(signature_data.sbytes, except_sbytes.to_vec());
    }

    use super::super::tez::generator::*;
    #[test]
    fn test_tez_generate() {
        let mnemonic_str = "burden mechanic acquire muscle flag home oil valley eager donor little essay daughter baby hover";
        let sk = "edskS2SDFgtTqWbEoKyW5CkXuwfki2NLzcGz6YDLQ5Pexp5iEuJgb8Wj6rG3D9pVrWRo9EJ4iihnqdvHx4cgSCGuTMjpSSSF7f";
        let pk = "edpkuw9X1bauTMKiAadWJJioLujYxADYrf4Q3dGRJbSmLbgGn3TC1j";
        let pkh = "tz1iJY251ptCJXWMdzhrkLNM72p2UZ48vuZb";
        let keypair = generate_keypair_from_mnemonic_str(mnemonic_str, "mWcziEO9fE8kzGsV");
        assert_eq!(sk, keypair.sk);
        assert_eq!(pk, keypair.pk);
        assert_eq!(pkh, keypair.pkh);
    }

    use bitcoin::util::base58;
    #[test]
    fn test_pkh() {
        let pk = "edpkvQQhHzGoFf2zSESp1Kh57sFhbhtA16XAGXqjAYse75BC9RdoXW";
        let except_pkh = "tz1MNu6ytbdEYrHyyQwctJ7rZVFcLrHWjKoN";
        let raw_pk_with_prefix = base58::from_check(pk).unwrap();
        let pkh = pkh_from_rawpk(&raw_pk_with_prefix[4..]);
        assert_eq!(except_pkh, pkh);
    }

    use super::super::tez::verify::*;
    #[test]
    fn test_tez_verify() {
        let test_data = "TEST".as_bytes();
        let sk = "edskS2SDFgtTqWbEoKyW5CkXuwfki2NLzcGz6YDLQ5Pexp5iEuJgb8Wj6rG3D9pVrWRo9EJ4iihnqdvHx4cgSCGuTMjpSSSF7f";
        let pk = base58::from_check("edpkuw9X1bauTMKiAadWJJioLujYxADYrf4Q3dGRJbSmLbgGn3TC1j").ok().unwrap();
        let signature_data = sign(test_data.to_vec(), sk);
        let (message, _) = preprocess(test_data.to_vec());
        assert!(verify(&message, &signature_data.sig, &pk[4..]));
    }
}
