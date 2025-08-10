## RUN THIS to test initrd
-  removed -bios
```shell
    /home/host/vtpm_guard/build/snp-release/usr/local/bin/qemu-system-x86_64 -enable-kvm -cpu EPYC-v4 -machine q35 -smp 1,maxcpus=255 -m 4096M -no-reboot -netdev user,id=vmnic,hostfwd=tcp:127.0.0.1:2222-:22,hostfwd=tcp:127.0.0.1:8080-:80  -device virtio-net-pci,disable-legacy=on,iommu_platform=true,netdev=vmnic,romfile= -drive file=/home/host/vtpm_guard/build/guest/sevsnptest.qcow2,if=none,id=disk0,format=qcow2 -device virtio-scsi-pci,id=scsi0,disable-legacy=on,iommu_platform=true -device scsi-hd,drive=disk0,bootindex=1 -kernel /home/host/vtpm_guard/build/kernel/boot/vmlinuz-6.9.0-snp-guest-a38297e3fb01 -append "console=ttyS0 earlyprintk=serial root=/dev/sda" -initrd /home/host/vtpm_guard/build/initramfs.cpio.gz -nographic -monitor pty -monitor unix:monitor,server,nowait -qmp tcp:localhost:4444,server,wait=off -chardev socket,id=chrtpm,path=/root/swtpm/swtpm-sock -tpmdev emulator,id=tpm0,chardev=chrtpm -device tpm-tis,tpmdev=tpm0
```


## GENERATING ENCRYPTED VM

- prepare host

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

- inside guest: 

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

- encrypt VM rootfs

```shell
make setup_luks
```