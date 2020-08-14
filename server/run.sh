#!/bin/bash
#make-run.sh
#make sure a process is always running.

export DISPLAY=:0 #needed if you are running a simple gui app.
dir=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
process=rust-risk
makerun="./target/release/rust-risk"

if ps ax | grep -v grep | grep $process > /dev/null
then
    exit
else
    $makerun > RustRisk.log &
fi

exit