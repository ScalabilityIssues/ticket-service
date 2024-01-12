use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tonic::{async_trait, Status};

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

    async fn list_tickets(&self) -> Result<Vec<Ticket>, Status> {
        self.ticket_collection()
            .find(None, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?
            .collect::<Result<Vec<Ticket>, _>>()
            .await
            .map_err(|e| Status::from_error(Box::new(e)))
    }

    async fn get_ticket(&self, id: ObjectId) -> Result<Ticket, Status> {
        self.ticket_collection()
            .find_one(doc! { "_id": &id }, None)
            .await
            .map_err(|_| Status::internal(""))?
            .ok_or(Status::not_found("ticket not found"))
    }

    async fn create_ticket(&self, ticket: Ticket) -> Result<ObjectId, Status> {
        self.ticket_collection()
            .insert_one(ticket, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))
            .map(|res| {
                res.inserted_id
                    .as_object_id()
                    .ok_or(Status::internal("idk"))
            })?
    }
}

impl TicketDatabase for Database {
    fn ticket_collection(&self) -> Collection<Ticket> {
        self.collection("tickets")
    }
}
