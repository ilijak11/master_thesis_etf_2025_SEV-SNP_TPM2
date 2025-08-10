## Tools and dependencies

* libtss2-dev
* tpm2-tools

# install dependencies:
```shell
sudo apt install libtss2-dev tmp2-tools
```

run [start_swtpm.sh](./start_swtpm.sh)

# test swtpm

run command
```shell
sudo tpm2_pcrread -T "swtpm:port=2321"
```