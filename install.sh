#!/bin/bash
function yes_or_no {
    while true; do
        read -p "$* [y/n]: " yn
        case $yn in
            [Yy]*) return 0  ;;  
            [Nn]*) echo "Aborted" ; return  1 ;;
        esac
    done
}


while getopts "bf:u" OPTION
do
	case $OPTION in
		b)
			echo "You set flag -b"
			exit
			;;
		f)
			echo "Building Features: $OPTARG"
			MYOPTF=$OPTARG
			if ! command -v cargo &> /dev/null
				then
    				echo "Cargo not installed. Install rust using curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly (y/n)"
					yes_or_no "$message" && curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
					echo "Rust has been installed. Re-run this command to build RR."
    				exit
				fi
			cargo build --release --features $OPTARG
			;;
		u)
			echo "Uninstalling . . ."
			sudo rm /usr/local/bin/rrringmaster
			sudo rm /usr/local/bin/rrserver
			echo "Uninstalled."
			exit
			;;
		\?)
			echo "Flags: u (uninstalls rust-risk), f (features, risk_image, risk_reddit, etc)"
			exit
			;;
	esac
done
echo $MYOPTF
if [ -e target/release/rrringmaster ]  && [ -e target/release/rrserver ]
then
    echo "Built. Installing . . . "
	sudo cp target/release/rrringmaster /usr/local/bin
    sudo cp target/release/rrserver /usr/local/bin
	sudo cp rustrisk.service /etc/systemd/system/
	echo "Installed."
else
    echo "Not built. Please run ./install -f \"(features you want)\" or cargo build --release --features \"(features you want)\" "
fi
exit