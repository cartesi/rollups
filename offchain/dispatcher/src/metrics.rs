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

use http_server::{CounterRef, Registry};

#[derive(Debug, Clone, Default)]
pub struct DispatcherMetrics {
    pub claims_sent: CounterRef,
    pub advance_inputs_sent: CounterRef,
    pub finish_epochs_sent: CounterRef,
}

impl From<DispatcherMetrics> for Registry {
    fn from(metrics: DispatcherMetrics) -> Self {
        let mut registry = Registry::default();
        registry.register(
            "claims_sent",
            "Counts the number of claims sent",
            metrics.claims_sent,
        );
        registry.register(
            "advance_inputs_sent",
            "Counts the number of <advance_input>s sent",
            metrics.advance_inputs_sent,
        );
        registry.register(
            "finish_epochs_sent",
            "Counts the number of <finish_epoch>s sent",
            metrics.finish_epochs_sent,
        );
        registry
    }
}
