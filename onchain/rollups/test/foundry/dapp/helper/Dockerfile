# (c) Cartesi and individual authors (see AUTHORS)
# SPDX-License-Identifier: Apache-2.0 (see LICENSE)

FROM cartesi/server-manager:0.8.2
WORKDIR /opt/gen-proofs

# root permission
USER root

# Install system dependencies
RUN apt-get update && \
    apt-get install -y wget xxd jq git python3 python3-pip python3-venv

# Create Python virtual environment
RUN python3 -m venv /opt/venv/

# Install Python dependencies
RUN /opt/venv/bin/pip install base64-to-hex-converter==0.4.1

# Install grpcurl
RUN export ARCH=$(uname -m | sed 's/aarch64/arm64/') && \
    export GRPCURL_VERSION=1.8.6 && \
    export GRPCURL_TARBALL=grpcurl_${GRPCURL_VERSION}_linux_${ARCH}.tar.gz && \
    wget https://github.com/fullstorydev/grpcurl/releases/download/v${GRPCURL_VERSION}/${GRPCURL_TARBALL} && \
    tar zxfv $GRPCURL_TARBALL && \
    rm $GRPCURL_TARBALL

# Download rootfs, linux and rom
ENV IMAGES_PATH /usr/share/cartesi-machine/images
RUN wget -O ${IMAGES_PATH}/rootfs.ext2 https://github.com/cartesi/image-rootfs/releases/download/v0.18.0/rootfs-v0.18.0.ext2 && \
    wget -O ${IMAGES_PATH}/linux.bin https://github.com/cartesi/image-kernel/releases/download/v0.17.0/linux-5.15.63-ctsi-2-v0.17.0.bin && \
    wget -O ${IMAGES_PATH}/rom.bin https://github.com/cartesi/machine-emulator-rom/releases/download/v0.17.0/rom-v0.17.0.bin

# Download grpc-interfaces repo
RUN git clone --branch v0.14.0 https://github.com/cartesi/grpc-interfaces

# Generate machine with echo and store it
ENV MACHINE_DIR=/tmp/gen-proofs-machine
RUN cartesi-machine \
    --ram-length=128Mi \
    --rollup \
    --store=$MACHINE_DIR \
    -- "ioctl-echo-loop --vouchers=1 --notices=1 --reports=1 --verbose=1"

CMD ["/bin/bash"]
