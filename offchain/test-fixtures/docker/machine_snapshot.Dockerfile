# (c) Cartesi and individual authors (see AUTHORS)
# SPDX-License-Identifier: Apache-2.0 (see LICENSE)

FROM cartesi/server-manager:0.8.0

USER root

# Install system dependencies
RUN apt update && \
    apt install -y wget

# Download rootfs, linux and rom
ENV IMAGES_PATH /opt/cartesi/share/images
RUN wget -O ${IMAGES_PATH}/rootfs.ext2 https://github.com/cartesi/image-rootfs/releases/download/v0.17.0/rootfs-v0.17.0.ext2 && \
    wget -O ${IMAGES_PATH}/linux.bin https://github.com/cartesi/image-kernel/releases/download/v0.16.0/linux-5.15.63-ctsi-2.bin && \
    wget -O ${IMAGES_PATH}/rom.bin https://github.com/cartesi/machine-emulator-rom/releases/download/v0.16.0/rom-v0.16.0.bin

# Generate machine with echo and store it
ENV SNAPSHOT_DIR=/opt/cartesi/share/dapp-bin
RUN cartesi-machine \
    --ram-length=128Mi \
    --rollup \
    --store=$SNAPSHOT_DIR \
    -- "ioctl-echo-loop --vouchers=1 --notices=1 --reports=1 --verbose=1"
