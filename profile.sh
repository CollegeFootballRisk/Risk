RUSTFLAGS="-C instrument-coverage" cargo test --release --all-features --bin rrringmaster;
grcov . --binary-path ./target/release/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html;
rm *.profraw