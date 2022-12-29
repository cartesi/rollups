# Copyright 2022 Cartesi Pte. Ltd.
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

FROM cartesi/server-manager:0.5.0

ARG user
ARG group
ARG uid
ARG gid

RUN if ! getent group ${gid}; then \
        groupadd -g ${gid} ${group}; \
    fi

RUN useradd -u ${uid} -g ${gid} -s /bin/sh -m ${user}

USER ${uid}:${gid}
