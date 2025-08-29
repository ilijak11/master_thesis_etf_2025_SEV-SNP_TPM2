* Building the OVMF
```shell
sudo apt install ovmf uuid-dev g++ nasm iasl

cd ~

git clone https://github.com/coconut-svsm/edk2.git
cd edk2/
git checkout svsm
git submodule init
git submodule update

export PYTHON3_ENABLE=TRUE
export PYTHON_COMMAND=python3
make -j16 -C BaseTools/
source ./edksetup.sh --reconfig
build -a X64 -b DEBUG -t GCC5 -D DEBUG_ON_SERIAL_PORT -D DEBUG_VERBOSE -DTPM2_ENABLE -p OvmfPkg/OvmfPkgX64.dsc

cp Build/OvmfX64/DEBUG_GCC5/FV/OVMF* ~/snp_vtpm_guard/ovmf
```

