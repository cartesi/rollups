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

use anyhow::Error;
use state_fold::Foldable;
use state_fold_types::ethers::prelude::{ContractError, Middleware};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct FoldableError(Error);

impl Display for FoldableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for FoldableError {}

impl From<Error> for FoldableError {
    fn from(error: Error) -> Self {
        Self(error)
    }
}

impl<M: Middleware + 'static> From<ContractError<M>> for FoldableError {
    fn from(contract_error: ContractError<M>) -> Self {
        FoldableError(contract_error.into())
    }
}

impl<M: Middleware + 'static, F: Foldable + 'static>
    From<state_fold::error::FoldableError<M, F>> for FoldableError
where
    <F as Foldable>::Error: Send + Sync,
{
    fn from(contract_error: state_fold::error::FoldableError<M, F>) -> Self {
        FoldableError(contract_error.into())
    }
}
