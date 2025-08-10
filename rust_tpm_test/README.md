# Tools and dependencies

* RUST (cargo)
* libtss2-dev
* tpm2-tools
* swtpm

## install dependencies:
```shell
sudo apt update
sudo apt install libtss2-dev tmp2-tools swtpm swtpm-tools
```

## install rust
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

run [start_swtpm.sh](./start_swtpm.sh)

## Run swtpm
```shell
sudo tpm2_pcrread -T "swtpm:port=2321"
```

## Run attestation workflow test
```shell
cargo run --bin gen_quote
```