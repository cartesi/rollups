use juniper::{GraphQLInputObject, GraphQLObject};

#[derive(GraphQLObject)]
#[graphql(description = "Connection pattern cursor based pagination page info")]
pub struct PageInfo {
    pub start_cursor: String,
    pub end_cursor: String,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[derive(GraphQLInputObject, Debug)]
pub struct NoticeKeys {
    pub session_id: Option<String>,
    pub epoch_index: Option<i32>,
    pub input_index: Option<i32>,
    pub notice_index: Option<i32>,
}

#[derive(GraphQLObject)]
#[graphql(
    description = "Notice generated during dapp advance request processing"
)]
pub struct Notice {
    pub id: i32,
    pub session_id: String,
    pub epoch_index: i32,
    pub input_index: i32,
    pub notice_index: i32,
    pub keccak: String,
    pub payload: String,
}

#[derive(GraphQLObject)]
pub struct NoticeEdge {
    pub node: Notice,
    pub cursor: String,
}

#[derive(GraphQLObject)]
pub struct NoticeConnection {
    pub total_count: i32,
    pub edges: Vec<NoticeEdge>,
    pub nodes: Vec<Notice>,
    pub page_info: PageInfo,
}

pub type Schema = juniper::RootNode<
    'static,
    super::queries::Query,
    juniper::EmptyMutation<super::queries::Context>,
    juniper::EmptySubscription<super::queries::Context>,
    juniper::DefaultScalarValue,
>;
