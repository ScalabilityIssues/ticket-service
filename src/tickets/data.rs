use mongodb::bson::{doc, oid::ObjectId, DateTime};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tokio_stream::StreamExt;
use tonic::async_trait;

use crate::errors::ApplicationError;
use crate::proto::ticketsrvc::TicketStatus;

type DbResult<T> = std::result::Result<T, ApplicationError>;

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub _id: ObjectId,
    pub url: String,
    pub flight_id: String,
    pub passenger: Passenger,
    pub reservation_datetime: DateTime,
    pub estimated_cargo_weight: u32,
    pub ticket_status: String,
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
    fn deleted_ticket_collection(&self) -> Collection<Ticket>;

    async fn list_tickets(
        &self,
        include_nonvalid: bool,
        flight_id: Option<&str>,
    ) -> DbResult<Vec<Ticket>> {
        let query = match flight_id {
            Some(flight_id) => doc! { "flight_id": doc! { "$eq": flight_id } },
            None => doc! {},
        };

        let stream_valid = self.ticket_collection().find(query, None).await?;
        let mut tickets = stream_valid.collect::<Result<Vec<_>, _>>().await?;
        if include_nonvalid {
            let stream_deleted = self.deleted_ticket_collection().find(None, None).await?;
            let deleted_tickets = stream_deleted.collect::<Result<Vec<_>, _>>().await?;
            tickets.extend(deleted_tickets);
        }
        Ok(tickets)
    }

    async fn get_ticket(&self, id: ObjectId, allow_nonvalid: bool) -> DbResult<Ticket> {
        let ticket = self
            .ticket_collection()
            .find_one(doc! { "_id": &id }, None)
            .await?;

        match ticket {
            Some(t) => Ok(t),
            None => {
                if allow_nonvalid {
                    let ticket = self
                        .deleted_ticket_collection()
                        .find_one(doc! { "_id": &id }, None)
                        .await?;
                    return ticket.ok_or_else(|| ApplicationError::not_found("ticket not found"));
                } else {
                    return Err(ApplicationError::not_found("ticket not found"));
                }
            }
        }
    }

    async fn get_ticket_from_url(&self, url: String, allow_nonvalid: bool) -> DbResult<Ticket> {
        let ticket = self
            .ticket_collection()
            .find_one(doc! { "url": &url }, None)
            .await?;

        match ticket {
            Some(t) => Ok(t),
            None => {
                if allow_nonvalid {
                    let ticket = self
                        .deleted_ticket_collection()
                        .find_one(doc! { "url": &url }, None)
                        .await?;
                    return ticket.ok_or_else(|| ApplicationError::not_found("ticket not found"));
                } else {
                    return Err(ApplicationError::not_found("ticket not found"));
                }
            }
        }
    }

    async fn create_ticket(&self, ticket: Ticket) -> DbResult<ObjectId> {
        let res = self.ticket_collection().insert_one(ticket, None).await?;
        let id = res.inserted_id.as_object_id().unwrap();
        Ok(id)
    }

    async fn delete_ticket(&self, id: ObjectId) -> DbResult<()> {
        // retrieve the ticket
        let mut ticket = self.get_ticket(id, false).await?;
        // set as invalid
        ticket.ticket_status = TicketStatus::Deleted.as_str_name().to_string();
        // insert the ticket in the deleted collection
        let _ = self
            .deleted_ticket_collection()
            .insert_one(ticket, None)
            .await?;
        // delete the ticket from the collection
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
                f => return Err(ApplicationError::invalid_update_path(f.to_string())),
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

    fn deleted_ticket_collection(&self) -> Collection<Ticket> {
        self.collection("tickets-deleted")
    }
}
