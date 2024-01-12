use mongodb::bson::{doc, oid::ObjectId, DateTime};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tonic::async_trait;

use crate::errors::DatabaseError;

type DbResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub _id: ObjectId,
    pub flight_id: String,
    pub passenger: Passenger,
    pub reservation_datetime: DateTime,
    pub estimated_cargo_weight: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Passenger {
    pub ssn: String,
    pub name: String,
    pub surname: String,
    pub birth_date: DateTime,
}

#[async_trait]
pub trait TicketDatabase {
    fn ticket_collection(&self) -> Collection<Ticket>;

    async fn list_tickets(&self) -> DbResult<Vec<Ticket>> {
        let stream = self.ticket_collection().find(None, None).await?;

        let tickets = stream.collect::<Result<Vec<_>, _>>().await?;

        Ok(tickets)
    }

    async fn get_ticket(&self, id: ObjectId) -> DbResult<Ticket> {
        let ticket = self
            .ticket_collection()
            .find_one(doc! { "_id": &id }, None)
            .await?;

        ticket.ok_or_else(|| DatabaseError::not_found("ticket not found"))
    }

    async fn create_ticket(&self, ticket: Ticket) -> DbResult<ObjectId> {
        let res = self.ticket_collection().insert_one(ticket, None).await?;
        let id = res.inserted_id.as_object_id().unwrap();
        Ok(id)
    }
}

impl TicketDatabase for Database {
    fn ticket_collection(&self) -> Collection<Ticket> {
        self.collection("tickets")
    }
}
