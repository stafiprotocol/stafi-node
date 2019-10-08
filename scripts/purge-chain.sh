#!/bin/bash
db=$1

if [[ "$OSTYPE" == "linux-gnu" ]]; then
  echo "Clearing local data from home dir: $HOME/.local/share/stafi"
	if [[ "$db" == "dev" ]]; then
		rm -rf ~/.local/share/stafi/chains/dev/db/
		rm -rf ~/.local/share/stafi/chains/development/db/
	elif [[ "$db" == "stafi" ]]; then
    	rm -rf ~/.local/share/stafi/chains/stafi/db/
    	rm -rf ~/.local/share/stafi/chains/stafi_testnet/db/
	else
		db="all"
	    rm -rf ~/.local/share/stafi/chains/dev/db/
	    rm -rf ~/.local/share/stafi/chains/development/db/
	    rm -rf ~/.local/share/stafi/chains/stafi/db/
	    rm -rf ~/.local/share/stafi/chains/stafi_testnet/db/
    	rm -rf ~/.local/share/stafi/chains/local_testnet/db/
	fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Clearing local data from home dir: $HOME/Library/Application Support/stafi"
	if [[ "$db" == "dev" ]]; then
		rm -rf ~/Library/Application\ Support/stafi/chains/dev/db/
		rm -rf ~/Library/Application\ Support/stafi/chains/development/db/
	elif [[ "$db" == "stafi" ]]; then
		rm -rf ~/Library/Application\ Support/stafi/chains/stafi/db/
		rm -rf ~/Library/Application\ Support/stafi/chains/stafi_testnet/db/
	else
		db="all"
		rm -rf ~/Library/Application\ Support/stafi/chains/dev/db/
		rm -rf ~/Library/Application\ Support/stafi/chains/development/db/
	    rm -rf ~/Library/Application\ Support/stafi/chains/stafi/db/
	    rm -rf ~/Library/Application\ Support/stafi/chains/stafi_testnet/db/
		rm -rf ~/Library/Application\ Support/stafi/chains/local_testnet/db/
	fi
else
  echo "Clearing local data from home dir: $HOME/.local/share/stafi"
	if [[ "$db" == "dev" ]]; then
		rm -rf ~/.local/share/stafi/chains/dev/db/
		rm -rf ~/.local/share/stafi/chains/development/db/
	elif [[ "$db" == "stafi" ]]; then
    	rm -rf ~/.local/share/stafi/chains/stafi/db/
    	rm -rf ~/.local/share/stafi/chains/stafi_testnet/db/
	else
		db="all"
	    rm -rf ~/.local/share/stafi/chains/dev/db/
	    rm -rf ~/.local/share/stafi/chains/development/db/
	    rm -rf ~/.local/share/stafi/chains/stafi/db/
	    rm -rf ~/.local/share/stafi/chains/stafi_testnet/db/
    	rm -rf ~/.local/share/stafi/chains/local_testnet/db/
	fi
fi

echo "Deleted $db databases"
