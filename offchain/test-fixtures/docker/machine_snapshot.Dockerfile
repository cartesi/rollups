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
