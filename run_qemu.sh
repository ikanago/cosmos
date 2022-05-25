#!/bin/bash

set -eu

EFI_BINARY=$1

qemu-img create -f raw disk.img 200M
mkfs.fat -n 'cosmos' -s 2 -f 2 -R 32 -F32 disk.img
mkdir -p mnt
sudo mount -o loop disk.img mnt
sudo mkdir -p mnt/EFI/BOOT
sudo cp "$EFI_BINARY" mnt/EFI/BOOT/"$EFI_BINARY"
sudo umount mnt

qemu-system-x86_64 \
    -drive if=pflash,file=/home/vscode/osbook/devenv/OVMF_CODE.fd \
    -drive if=pflash,file=/home/vscode/osbook/devenv/OVMF_VARS.fd \
    -hda disk.img \
    -vnc :0
