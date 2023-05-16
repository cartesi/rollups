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

use std::cmp::Ordering;
use std::iter::Peekable;

pub struct MergeAscending<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
}

impl<L, R> MergeAscending<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
{
    pub fn new(left: L, right: R) -> Self {
        MergeAscending {
            left: left.peekable(),
            right: right.peekable(),
        }
    }
}

impl<L, R> Iterator for MergeAscending<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
    L::Item: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<L::Item> {
        let which = match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) => Some(l.cmp(r)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => None,
        };

        match which {
            Some(Ordering::Less) => self.left.next(),
            Some(Ordering::Equal) => self.left.next(),
            Some(Ordering::Greater) => self.right.next(),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (ll, lu) = self.left.size_hint();
        let (rl, ru) = self.right.size_hint();

        let l = ll + rl;
        let u = match (lu, ru) {
            (Some(l), Some(r)) => Some(l + r),
            _ => None,
        };

        (l, u)
    }
}
