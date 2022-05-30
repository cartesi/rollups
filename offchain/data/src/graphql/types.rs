use juniper::{
    graphql_scalar,
    parser::{ParseError, ScalarToken, Token},
    GraphQLInputObject, GraphQLObject, GraphQLScalarValue, ParseScalarResult,
    Value, ID,
};
use std::cmp::Ordering;

/// Helper macro to implement partial order related traits based on id
macro_rules! implement_ordering {
    ($cursor_type:ty) => {
        impl PartialOrd for $cursor_type {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                let id_i32 = self.id.parse::<i32>().unwrap_or_default();
                let other_i32 = other.id.parse::<i32>().unwrap_or_default();
                id_i32.partial_cmp(&other_i32)
            }
        }

        impl PartialEq for $cursor_type {
            fn eq(&self, other: &Self) -> bool {
                (*self.id).eq(&*other.id)
            }
        }
        impl Eq for $cursor_type {}

        impl Ord for $cursor_type {
            fn cmp(&self, other: &Self) -> Ordering {
                let id_i32 = self.id.parse::<i32>().unwrap_or_default();
                let other_i32 = other.id.parse::<i32>().unwrap_or_default();
                id_i32.cmp(&other_i32)
            }
        }
    };
}

/// Custom Graphql scalar definition, to be able to use long (signed 64)
/// values
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

#[derive(GraphQLObject, Debug, Clone)]
#[graphql(description = "Connection pattern cursor based pagination page info")]
pub struct PageInfo {
    pub start_cursor: String,
    pub end_cursor: String,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[derive(Debug, Clone)]
pub struct Epoch {
    pub id: ID,
    pub index: i32,
}

implement_ordering!(Epoch);

#[derive(Debug, Clone)]
pub struct EpochEdge {
    pub node: Epoch,
    pub cursor: String,
}

#[derive(Debug, Clone)]
pub struct EpochConnection {
    pub total_count: i32,
    pub edges: Vec<EpochEdge>,
    pub nodes: Vec<Epoch>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub(super) id: juniper::ID,
    pub(super) index: i32,
    pub(super) epoch: Epoch,
    pub(super) msg_sender: String,
    pub(super) tx_hash: Option<String>,
    pub(super) timestamp: i64,
    pub(super) block_number: i64,
}

implement_ordering!(Input);

#[derive(Debug, Clone)]
pub struct InputEdge {
    pub(super) node: Input,
    pub(super) cursor: String,
}

#[derive(Debug, Clone)]
pub struct InputConnection {
    pub total_count: i32,
    pub edges: Vec<InputEdge>,
    pub nodes: Vec<Input>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, GraphQLInputObject)]
#[graphql(scalar = RollupsGraphQLScalarValue)]
pub struct InputFilter {
    th_hash: Option<String>,
    msg_sender: Option<String>,
    block_number: Option<i64>,
    block_number_lower_than: Option<i64>,
    block_number_greater_than: Option<i64>,
    timestamp_lower_than: Option<i64>,
    timestamp_greater_than: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub id: juniper::ID,
    pub index: i32,
    pub input: Input,
    pub proof: Option<Proof>,
    pub keccak: String,
    pub payload: String,
}

implement_ordering!(Notice);

#[derive(Debug, Clone)]
pub struct NoticeEdge {
    pub node: Notice,
    pub cursor: String,
}

#[derive(Debug, Clone)]
pub struct NoticeConnection {
    pub total_count: i32,
    pub edges: Vec<NoticeEdge>,
    pub nodes: Vec<Notice>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, GraphQLInputObject)]
pub struct NoticeFilter {
    dummy: String,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub id: juniper::ID,
    pub index: i32,
    pub input: Input,
    pub payload: String,
}

implement_ordering!(Report);

#[derive(Debug, Clone)]
pub struct ReportEdge {
    pub node: Report,
    pub cursor: String,
}

#[derive(Debug, Clone)]
pub struct ReportConnection {
    pub total_count: i32,
    pub edges: Vec<ReportEdge>,
    pub nodes: Vec<Report>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, GraphQLInputObject)]
pub struct ReportFilter {
    dummy: String,
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub output_hashes_root_hash: String,
    pub vouchers_epoch_root_hash: String,
    pub notices_epoch_root_hash: String,
    pub machine_state_hash: String,
    pub keccak_in_hashes_siblings: Vec<String>,
    pub output_hashes_in_epoch_siblings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Voucher {
    pub id: juniper::ID,
    pub index: i32,
    pub input: Input,
    pub proof: Option<Proof>,
    pub destination: String,
    pub payload: String,
}

#[derive(Debug, Clone)]
pub struct VoucherEdge {
    pub node: Voucher,
    pub cursor: String,
}

#[derive(Debug, Clone)]
pub struct VoucherConnection {
    pub total_count: i32,
    pub edges: Vec<VoucherEdge>,
    pub nodes: Vec<Voucher>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, GraphQLInputObject)]
pub struct VoucherFilter {
    destination: String,
}

pub struct Query;

pub type Schema = juniper::RootNode<
    'static,
    super::types::Query,
    juniper::EmptyMutation<super::resolvers::Context>,
    juniper::EmptySubscription<super::resolvers::Context>,
    RollupsGraphQLScalarValue,
>;
