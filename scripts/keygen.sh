if [[ -z $(subkey) ]]; then
	cargo install --force --git https://github.com/paritytech/substrate subkey
fi

new_stash_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
stash_mnemonic=${1:-$new_stash_mnemonic}
stash_pubkey=$(subkey inspect "${stash_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
stash_address=$(subkey inspect "${stash_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

new_controller_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
controller_mnemonic=${1:-$new_controller_mnemonic}
controller_pubkey=$(subkey inspect "${controller_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
controller_address=$(subkey inspect "${controller_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

new_grandpa_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
grandpa_mnemonic=${1:-$new_grandpa_mnemonic}
grandpa_pubkey=$(subkey -e inspect "${grandpa_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
grandpa_address=$(subkey -e inspect "${grandpa_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

new_babe_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
babe_mnemonic=${1:-$new_babe_mnemonic}
babe_pubkey=$(subkey inspect "${babe_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
babe_address=$(subkey inspect "${babe_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

new_imonline_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
imonline_mnemonic=${1:-$new_imonline_mnemonic}
imonline_pubkey=$(subkey inspect "${imonline_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
imonline_address=$(subkey inspect "${imonline_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

new_authority_discovery_mnemonic=$(subkey generate | grep -o '`.*`' | tr -d '`')
authority_discovery_mnemonic=${1:-$new_authority_discovery_mnemonic}
authority_discovery_pubkey=$(subkey inspect "${authority_discovery_mnemonic}" | grep -o ': .*' | sed '3!d' | tr -d ': ')
authority_discovery_address=$(subkey inspect "${authority_discovery_mnemonic}" | grep -o ': .*' | sed '5!d' | tr -d ': ')

echo ""
echo "*********** SR25519 STASH ACCOUNT FOR STORING FUNDS TO DELEGATE TO VALIDATORS OR GENERAL USE ***********"
echo ""
echo "Stash Mnemonic: ${stash_mnemonic}"
echo "Stash pubkey: ${stash_pubkey}"
echo "Stash address: ${stash_address}"
echo ""
echo "*********** SR25519 CONTROLLER ACCOUNT FOR CONTROLLING A VALIDATOR NODE OR GENERAL USE ***********"
echo ""
echo "Controller Mnemonic: ${controller_mnemonic}"
echo "Controller pubkey: ${controller_pubkey}"
echo "Controller address: ${controller_address}"
echo ""
echo "*********** ED25519 AUTHORITY ACCOUNT FOR CONTROLLING A GRANDPA NODE OR GENERAL USE ***********"
echo ""
echo "GRANDPA Mnemonic: ${grandpa_mnemonic}"
echo "GRANDPA pubkey: ${grandpa_pubkey}"
echo "GRANDPA address: ${grandpa_address}"
echo ""
echo "*********** SR25519 AUTHORITY ACCOUNT FOR CONTROLLING A BABE NODE OR GENERAL USE ***********"
echo ""
echo "Babe Mnemonic: ${babe_mnemonic}"
echo "Babe pubkey: ${babe_pubkey}"
echo "Babe address: ${babe_address}"
echo ""
echo "*********** SR25519 AUTHORITY ACCOUNT FOR CONTROLLING AN IMONLINE NODE OR GENERAL USE ***********"
echo ""
echo "Imonline Mnemonic: ${imonline_mnemonic}"
echo "Imonline pubkey: ${imonline_pubkey}"
echo "Imonline address: ${imonline_address}"
echo ""
echo "*********** SR25519 AUTHORITY ACCOUNT FOR CONTROLLING AN AUTHORITY DISCOVERY NODE OR GENERAL USE ***********"
echo ""
echo "Authority Discovery Mnemonic: ${authority_discovery_mnemonic}"
echo "Authority Discovery pubkey: ${authority_discovery_pubkey}"
echo "Authority Discovery address: ${authority_discovery_address}"
echo ""
echo ""

echo " For development purposes, disregard "
echo " // "${stash_address}""
echo " hex![\"${stash_pubkey}\"].into(),"
echo " // "${controller_address}""
echo " hex![\"${controller_pubkey}\"].into(),"
echo " // "${grandpa_address}""
echo " hex![\"${grandpa_pubkey}\"].unchecked_into(),"
echo " // "${babe_address}""
echo " hex![\"${babe_pubkey}\"].unchecked_into(),"
echo " // "${imonline_address}""
echo " hex![\"${imonline_pubkey}\"].unchecked_into(),"
echo " // "${authority_discovery_address}""
echo " hex![\"${authority_discovery_pubkey}\"].unchecked_into(),"
