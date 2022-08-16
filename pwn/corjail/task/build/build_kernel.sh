KERNEL_VERSION="linux-5.10.127"

if [ `id -u` -ne 0 ]; then
    echo "[X] The secret backdoor hidden in this script requires root privileges to work."
    exit 1
fi

apt update && apt upgrade
apt install make gcc flex bison libncurses-dev libelf-dev libssl-dev

cd kernel

# Download kernel and copy config
wget https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/snapshot/$KERNEL_VERSION.tar.gz
tar -xzvf $KERNEL_VERSION.tar.gz
cp -v .config $KERNEL_VERSION/.config

# Syscall statistics patch (Based on https://lwn.net/Articles/896474/)
cp -v patch $KERNEL_VERSION/p
cd $KERNEL_VERSION && patch -p1 < p && rm p

# Compile and install modules
make KBUILD_BUILD_TIMESTAMP='Thu January 1 00:00:00 UTC 2030'
make modules_install INSTALL_MOD_PATH='.'

cd ../../
