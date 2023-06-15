// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

mod common;

mod grpc_tests {
    mod advance_state;
    mod delete_epoch;
    mod end_session;
    mod finish_epoch;
    mod get_epoch_status;
    mod get_session_status;
    mod get_status;
    mod get_version;
    mod inspect_state;
    mod start_session;
}
