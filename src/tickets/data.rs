use mongodb::bson::{doc, oid::ObjectId, DateTime};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tokio_stream::StreamExt;
use tonic::async_trait;

use crate::errors::DatabaseError;

type DbResult<T> = std::result::Result<T, DatabaseError>;

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub _id: ObjectId,
    pub url: String,
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
    pub email: String,
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

    async fn delete_ticket(&self, id: ObjectId) -> DbResult<()> {
        self.ticket_collection()
            .delete_one(doc! { "_id": &id }, None)
            .await?;

        Ok(())
    }

    async fn update_ticket(
        &self,
        id: ObjectId,
        update: Ticket,
        update_paths: BTreeSet<String>,
    ) -> DbResult<()> {
        let mut updated_doc = doc! {};

        let Ticket { passenger, .. } = update;

        for field in update_paths {
            match field.as_str() {
                "passenger.ssn" => updated_doc.insert(field, passenger.ssn.clone()),
                "passenger.name" => updated_doc.insert(field, passenger.name.clone()),
                "passenger.surname" => updated_doc.insert(field, passenger.surname.clone()),
                "passenger.birth_date" => updated_doc.insert(field, passenger.birth_date.clone()),
                "passenger.email" => updated_doc.insert(field, passenger.email.clone()),
                f => return Err(DatabaseError::invalid_update_path(f.to_string())),
            };
        }

        self.ticket_collection()
            .update_one(doc! { "_id": &id }, doc! { "$set": updated_doc }, None)
            .await?;

        Ok(())
    }

    async fn get_existing_tickets(&self, flight_id: &str) -> DbResult<u32> {
        let count = self
            .ticket_collection()
            .count_documents(doc! { "flight_id": flight_id }, None)
            .await?;

        Ok(count.try_into().unwrap())
    }
}

impl TicketDatabase for Database {
    fn ticket_collection(&self) -> Collection<Ticket> {
        self.collection("tickets")
    }
}
