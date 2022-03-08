// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { task } from "hardhat/config";
import { NoticesArgs } from "./args";
import { taskDefs, TASK_GET_NOTICES } from "./constants";
import { connected } from "./graphql";
import { noticesParams, graphqlParams } from "./params";
import { GetNoticeDocument } from "../generated/graphql";
import { ethers } from "ethers";

graphqlParams(
    noticesParams(
        task<NoticesArgs>(
            TASK_GET_NOTICES,
            taskDefs[TASK_GET_NOTICES].description,
            connected(async (args, client) => {
                const { data, error } = await client
                    .query(GetNoticeDocument, {
                        query: {
                            epoch_index: args.epoch.toString(),
                        },
                    })
                    .toPromise();

                if (error) {
                    console.error(error.message);
                    return;
                }

                data?.GetNotice?.forEach((notice) => {
                    if (notice) {
                        delete notice.__typename;
                        if (args.payload == "string") {
                            // converts payload from hex to string format
                            notice.payload = "0x" + notice.payload;
                            try {
                                notice.payload = ethers.utils.toUtf8String(
                                    notice.payload
                                );
                            } catch (e) {
                                console.error(
                                    `Error converting hex string '${notice.payload}'`
                                );
                            }
                        }
                        console.log(JSON.stringify(notice));
                    }
                });
            })
        )
    )
);
