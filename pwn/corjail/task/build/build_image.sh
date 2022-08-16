#!/bin/bash

FS=corfs
IMAGE="coros/coros.img"
IMAGE_QCOW2="coros/coros.qcow2"

if [ `id -u` -ne 0 ]; then
    echo "[X] The secret backdoor hidden in this script requires root privileges to work."
    exit 1
fi 

# Cleanup
rm -rf $FS $IMAGE
rm -rf /mnt/$FS

# Update packages
apt update && apt upgrade

# Install requirements
apt install debootstrap qemu-system-x86 qemu-utils libguestfs-tools

# Create Debian image
debootstrap \
    --arch=amd64 \
    --variant=minbase \
    --components=main,contrib,non-free \
    --include=apparmor,docker.io,ifupdown,init,isc-dhcp-client,udev \
    --exclude=apt,e2fsprogs,gcc-9-base,perl-base \
    bullseye $FS

# Copy shared files
cp coros/files/config/serial-getty@.service $FS/lib/systemd/system/serial-getty@.service
cp coros/files/config/init.service $FS/etc/systemd/system/init.service
cp coros/files/bin/init $FS/usr/local/bin/init
cp coros/files/docker/seccomp.json $FS/etc/docker/corjail.json

# Copy kernel module
mkdir -p $FS/lib/modules/5.10.127
cp coros/files/module/* $FS/lib/modules/5.10.127

# Copy build-specific files
cp coros/files/bin/jail $FS/usr/local/bin/jail
cp coros/files/flag.txt $FS/root/temp

# Copy docker image
tar -xzvf coros/files/docker/image/image.tar.gz -C coros/files/docker
cp -rp coros/files/docker/var/lib/docker $FS/var/lib/
rm -rf coros/files/docker/var

# Fix iptables and enable challenge initialization service
chroot $FS/ sh -c "update-alternatives --set iptables /usr/sbin/iptables-legacy"
chroot $FS/ sh -c "update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy"
chroot $FS/ sh -c "systemctl enable init.service"

# FS/Networking stuff
echo -en '\nauto eth0\niface eth0 inet dhcp\n' | tee $FS/etc/network/interfaces
echo -en "127.0.0.1\tlocalhost\n" | tee $FS/etc/hosts
echo 'T0:23:respawn:/sbin/getty -L ttyS0 115200 vt100' | tee $FS/etc/inittab
echo '/dev/root / ext4 defaults 0 0' | tee $FS/etc/fstab
echo 'nameserver 8.8.8.8' | tee $FS/etc/resolve.conf
echo 'CoROS' | tee $FS/etc/hostname
echo 'cormon' | tee $FS/etc/modules
sed -i 's/root:x:0:0:root:\/root:\/bin\/bash/root:x:0:0:root:\/root:\/usr\/local\/bin\/jail/g' $FS/etc/passwd
rm $FS/etc/update-motd.d/*
cat /dev/null > $FS/etc/motd
cp coros/files/config/motd `find $FS/var/lib/ -name motd | tail -n 1`

# Cleanup logs
for x in `find $FS/var/log -type f`; do cat /dev/null > $x; done
for x in `find $FS/`; do touch -t 0606060606 $x; done

# Build image
dd if=/dev/zero of=$IMAGE bs=1024 seek=2047 count=1024000
mkfs.ext4 -F $IMAGE
mkdir -p /mnt/$FS
mount -o loop $IMAGE /mnt/$FS
cp -a $FS/. /mnt/$FS/.
umount /mnt/$FS

# Convert to qcow2
qemu-img convert -c -O qcow2 $IMAGE $IMAGE_QCOW2.tmp
virt-sparsify $IMAGE_QCOW2.tmp --compress $IMAGE_QCOW2

# Cleanup
rm -rf $FS $IMAGE $IMAGE_QCOW2.tmp
rm -rf /mnt/$FS

echo "[+] Image successfully created in $IMAGE_QCOW2"
