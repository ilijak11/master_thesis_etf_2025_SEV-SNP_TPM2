#!/bin/bash
mkdir -p ./swtpm_test/tpm

sudo swtpm_setup \
    --tpm-state ./swtpm_test/tpm/ \
    --tpm2 \
    --create-ek-cert \
    --create-platform-cert \
    --lock-nvram

swtpm socket \
  --tpm2 \
  --tpmstate dir=./swtpm_test/tpm \
  --ctrl type=tcp,port=2322 \
  --server type=tcp,port=2321 \
  --flags not-need-init,startup-clear \
  --log level=20
