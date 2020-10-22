use anyhow::Result;
use tokio::net::ToSocketAddrs;

use crate::frame::response::Response;
use crate::query::Query;
use crate::prepared_statement::PreparedStatement;
use crate::transport::connection::Connection;

pub struct Session {
    connection: Connection,
}

impl Session {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let connection = Connection::new(addr).await?;

        connection.startup(Default::default()).await?;

        Ok(Session { connection })
    }

    // TODO: Should return an iterator over results
    pub async fn query(&self, query: impl Into<Query>) -> Result<()> {
        let result = self.connection.query(&query.into()).await?;
        match result {
            Response::Error(err) => {
                return Err(err.into());
            }
            Response::Result(_) => {}
            _ => return Err(anyhow!("Unexpected frame received")),
        }
        Ok(())
    }

    pub async fn prepare(&self, query: String) -> Result<PreparedStatement> {
        let result = self.connection.prepare(query).await?;
        match result {
            Response::Error(err) => {
                Err(err.into())
            }
            Response::Result(_) => {
                //FIXME: actually read the id
                Ok(PreparedStatement::new("stub_id".into()))
            }
            _ => return Err(anyhow!("Unexpected frame received")),
        }
    }
}
