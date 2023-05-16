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
//
// Parts of the code (BigInt scalar implementatation) is licenced
// under BSD 2-Clause Copyright (c) 2016, Magnus Hallin

use juniper::parser::{ParseError, ScalarToken, Token};
use juniper::{
    graphql_scalar, GraphQLScalarValue, ParseScalarResult, ScalarValue, Value,
};

/// Custom Graphql scalar definition, to be able to use long (signed 64) values
#[derive(Debug, Clone, PartialEq, GraphQLScalarValue)]
pub enum RollupsGraphQLScalarValue {
    Int(i32),
    BigInt(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[graphql_scalar(name = "BigInt")]
impl GraphQLScalar for i64 {
    fn resolve(&self) -> Value {
        Value::scalar(*self)
    }

    fn from_input_value(v: &juniper::InputValue) -> Option<i64> {
        v.as_scalar_value::<i64>().copied()
    }

    fn from_str<'a>(
        value: ScalarToken<'a>,
    ) -> ParseScalarResult<'a, RollupsGraphQLScalarValue> {
        if let ScalarToken::Int(v) = value {
            v.parse()
                .map_err(|_| ParseError::UnexpectedToken(Token::Scalar(value)))
                .map(|s: i64| s.into())
        } else {
            Err(ParseError::UnexpectedToken(Token::Scalar(value)))
        }
    }
}

impl ScalarValue for RollupsGraphQLScalarValue {
    type Visitor = RollupsGraphQLScalarValueVisitor;

    fn as_int(&self) -> Option<i32> {
        match *self {
            Self::Int(ref i) => Some(*i),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match *self {
            Self::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match *self {
            Self::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f64> {
        match *self {
            Self::Int(ref i) => Some(*i as f64),
            Self::Float(ref f) => Some(*f),
            _ => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match *self {
            Self::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct RollupsGraphQLScalarValueVisitor;

impl<'de> serde::de::Visitor<'de> for RollupsGraphQLScalarValueVisitor {
    type Value = RollupsGraphQLScalarValue;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("a valid input value")
    }

    fn visit_bool<E>(
        self,
        value: bool,
    ) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::Boolean(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        Ok(RollupsGraphQLScalarValue::Int(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i32::max_value() as i64 {
            self.visit_i32(value as i32)
        } else {
            Ok(RollupsGraphQLScalarValue::BigInt(value))
        }
    }

    fn visit_u32<E>(self, value: u32) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i32::max_value() as u32 {
            self.visit_i32(value as i32)
        } else {
            self.visit_u64(value as u64)
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i64::MAX as u64 {
            self.visit_i64(value as i64)
        } else {
            Ok(RollupsGraphQLScalarValue::Float(value as f64))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(value.into())
    }

    fn visit_string<E>(
        self,
        value: String,
    ) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::String(value))
    }
}
