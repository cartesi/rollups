# Copyright Cartesi Pte. Ltd.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

FROM cartesi/server-manager:0.7.0
WORKDIR /opt/gen-proofs

# root permission
USER root

# Install system dependencies
RUN apt-get update && \
    apt-get install -y wget xxd jq git

# Install grpcurl
RUN export ARCH=$(uname -m | sed 's/aarch64/arm64/') && \
    export GRPCURL_VERSION=1.8.6 && \
    export GRPCURL_TARBALL=grpcurl_${GRPCURL_VERSION}_linux_${ARCH}.tar.gz && \
    wget https://github.com/fullstorydev/grpcurl/releases/download/v${GRPCURL_VERSION}/${GRPCURL_TARBALL} && \
    tar zxfv $GRPCURL_TARBALL && \
    rm $GRPCURL_TARBALL

# Download rootfs, linux and rom
ENV IMAGES_PATH /opt/cartesi/share/images
RUN wget -O ${IMAGES_PATH}/rootfs.ext2 https://github.com/cartesi/image-rootfs/releases/download/v0.15.0/rootfs-v0.15.0.ext2 && \
    wget -O ${IMAGES_PATH}/linux.bin https://github.com/cartesi/image-kernel/releases/download/v0.14.0/linux-5.15.63-ctsi-1.bin && \
    wget -O ${IMAGES_PATH}/rom.bin https://github.com/cartesi/machine-emulator-rom/releases/download/v0.13.0/rom-v0.13.0.bin

# Download grpc-interfaces repo
RUN git clone --branch v0.12.0 https://github.com/cartesi/grpc-interfaces

# Generate machine with echo and store it
ENV MACHINE_DIR=/tmp/gen-proofs-machine
RUN cartesi-machine \
    --ram-length=128Mi \
    --rollup \
    --store=$MACHINE_DIR \
    -- "ioctl-echo-loop --vouchers=1 --notices=1 --reports=1 --verbose=1"

CMD ["/bin/bash"]
