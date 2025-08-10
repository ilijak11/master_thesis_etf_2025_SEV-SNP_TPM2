# CVM attestation AMD SEV-SNP + TPM 2.0
This project aims to add additional attestation method using vTPM to existing project: [SNPGuard](https://github.com/SNPGuard/snp-guard), which implements AMD SEV-SNP attestation on CVMs.
SNPGuard provides tools (scripts) for building all necessary components for SEV-SNP attestation such as: 
* host kernel modifications, 
* guest kernel modifications, 
* qemu/kvm modifications, 
* Makefile to run commands for instantiating CVMs

Complete SNPGuard documentation can be found [here](https://github.com/SNPGuard/snp-guard/blob/main/README.md).

This project provides modifications to enumerated tools and furher adds **vTPM** attestation capability to existing RUST attestation binaries to **initramfs**
The vTPM is emulated in guest using **swtpm**. Initramfs has access to /dev/tpm0 and is able to generate a quote over PCR values.


Following are steps to createa and run CVMs in various modes:

## Install dependencies
* install dependencies by running:
```shell
./install-dependencies.sh
```

* additionally install swtpm to emulate vTPM inside guest
```shell
sudo apt update
sudo apt install libtss2-dev tmp2-tools swtpm swtpm-tools
```

## GENERATING VM
* prepare host
```shell
mkdir -p build && cd build
wget https://github.com/SNPGuard/snp-guard/releases/download/v0.1.2/snp-release.tar.gz
tar -xf snp-release.tar.gz
cd ..

# Unpack kernel to ./build/kernel
make unpack_kernel

# Create initramfs
make initramfs

# create image (will be stored under build/guest/sevsnptest.qcow2)
make create_new_vm
## sevsnpvm login: guest
## Password: password

# run image for initial setup
# Note: if you don't see a prompt after cloud-init logs, press ENTER
make run_setup
## login with credentials

# (From another shell) Copy kernel and headers to the guest VM via SCP
# note: if the guest does not have an IP address check below instructions
scp -P 2222 build/snp-release/linux/guest/*.deb guest@localhost:
## if "REMOTE HOST IDENTIFICATION CHANGED" error:
## rm -rf ~/.ssh/known_hosts ## only if no other hosts 
```

* inside guest: 
```shell
# install kernel and headers (copied before)
# This is needed even when running direct boot, as we still need access to the kernel module files
sudo dpkg -i linux-*.deb

# remove kernel and headers to save space
rm -rf linux-*.deb

# disable multipath service (causes some conflicts)
sudo systemctl disable multipathd.service

# Shut down VM
sudo shutdown now
```

* prepare VM config template
```shell
make fetch_vm_config_template
```


## GENERATING AND RUNNING DIRRECT BOOT (custom initramfs)
* run encrypted boot option (no attestation)
```shell
make  make run_direct_boot
```

## GENERATING AND RUNNING ENCRYPTED VM
* create image with encrypted rootfs
```shell
make setup_luks
#password: password
```

* run encrypted boot option (no attestation)
```shell
make run_encrypted_rootfs_boot
```
