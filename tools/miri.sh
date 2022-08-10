export RUST_BACKTRACE=1
export MIRIFLAGS="-Zmiri-disable-isolation"
cargo +nightly miri test