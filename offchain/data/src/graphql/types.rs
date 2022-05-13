use juniper::{GraphQLObject, GraphQLScalarValue, ID};
use std::cmp::Ordering;

/// Custom Graphql scalar definition, to be able to use long (signed 64)
/// values
#[derive(Debug, Clone, PartialEq, GraphQLScalarValue)]
pub enum CartesiGraphQLScalarValue {
    Int(i32),
    Long(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(GraphQLObject, Debug)]
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

impl PartialOrd for Epoch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.partial_cmp(&other_i32)
    }
}
impl PartialEq for Epoch {
    fn eq(&self, other: &Self) -> bool {
        (*self.id).eq(&*other.id)
    }
}
impl Eq for Epoch {}

impl Ord for Epoch {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.cmp(&other_i32)
    }
}

#[derive(Debug)]
pub struct EpochEdge {
    pub node: Epoch,
    pub cursor: String,
}

#[derive(Debug)]
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
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.partial_cmp(&other_i32)
    }
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.id != other.id
    }
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.cmp(&other_i32)
    }
}

impl Eq for Input {}

#[derive(Debug)]
pub struct InputEdge {
    pub(super) node: Input,
    pub(super) cursor: String,
}

#[derive(Debug)]
pub struct InputConnection {
    pub total_count: i32,
    pub edges: Vec<InputEdge>,
    pub nodes: Vec<Input>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub id: juniper::ID,
    pub index: i32,
    pub session_id: String,
    pub input: Input,
    pub keccak: String,

    pub payload: String,
}

impl PartialOrd for Notice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.partial_cmp(&other_i32)
    }
}
impl PartialEq for Notice {
    fn eq(&self, other: &Self) -> bool {
        (*self.id).eq(&*other.id)
    }
}
impl Eq for Notice {}

impl Ord for Notice {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_i32 = self.id.parse::<i32>().unwrap_or_default();
        let other_i32 = other.id.parse::<i32>().unwrap_or_default();
        id_i32.cmp(&other_i32)
    }
}

#[derive(Debug)]
pub struct NoticeEdge {
    pub node: Notice,
    pub cursor: String,
}

#[derive(Debug)]
pub struct NoticeConnection {
    pub total_count: i32,
    pub edges: Vec<NoticeEdge>,
    pub nodes: Vec<Notice>,
    pub page_info: PageInfo,
}

pub struct Query;

pub type Schema = juniper::RootNode<
    'static,
    super::types::Query,
    juniper::EmptyMutation<super::resolvers::Context>,
    juniper::EmptySubscription<super::resolvers::Context>,
    CartesiGraphQLScalarValue,
>;
