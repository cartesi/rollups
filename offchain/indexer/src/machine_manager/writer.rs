use crate::error::*;
use crate::machine_manager::db::{
    DbEpochStatusResponse, DbSessionStatusResponse,
};

use crate::grpc::{
    server_manager::{
        GetEpochStatusResponse, GetSessionStatusResponse, GetStatusResponse,
    },
    versioning::GetVersionResponse,
};

use crate::db::PollingPool;

#[derive(Clone)]
pub struct Writer {
    pub pool: PollingPool,
}

impl Writer {
    pub async fn consume_version(
        &self,
        _response: &GetVersionResponse,
    ) -> Result<()> {
        //TODO Once we have the Version as a type in GraphQL,
        //handle the response correctly

        Ok(())
    }
    pub async fn consume_status(
        &self,
        _response: &GetStatusResponse,
    ) -> Result<()> {
        //TODO Once we have the Status as a type in GraphQL,
        //handle the response correctly

        Ok(())
    }

    pub async fn consume_session_status(
        &self,
        response: &GetSessionStatusResponse,
    ) -> Result<()> {
        let my_session_status = DbSessionStatusResponse::from(response.clone());
        my_session_status.insert(&self.pool)?;

        Ok(())
    }

    pub async fn consume_epoch_status(
        &self,
        response: &GetEpochStatusResponse,
    ) -> Result<()> {
        let mut response_db = DbEpochStatusResponse::from(response.clone());
        response_db.insert(&self.pool)?;

        Ok(())
    }
}
