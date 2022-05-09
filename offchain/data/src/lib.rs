/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

pub mod database;
pub mod graphql;

#[macro_use]
extern crate diesel;

/// Create new backoff library error based on error that happened
pub fn new_backoff_err<E: std::fmt::Display>(err: E) -> backoff::Error<E> {
    // Retry according to backoff policy
    backoff::Error::Transient {
        err,
        retry_after: None,
    }
}
