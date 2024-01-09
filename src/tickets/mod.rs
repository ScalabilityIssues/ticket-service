use mongodb::bson::DateTime;
use mongodb::{
    bson::{
        doc,
        serde_helpers::{
            serialize_bson_datetime_as_rfc3339_string, serialize_hex_string_as_object_id,
        },
    },
    Collection, Database,
};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

use crate::proto::ticketmngr::PassengerDetails;
use crate::proto::ticketmngr::{
    tickets_server::Tickets, Ticket, TicketList, TicketQuery, TicketUpdate,
};

#[derive(Debug)]
pub struct TicketsApp {
    mongo_client: Database,
}

#[derive(Serialize, Deserialize)]
pub struct Ticket1 {
    #[serde(serialize_with = "serialize_hex_string_as_object_id")]
    _id: String,
    flight_id: String,
    passenger: Passenger,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    reservation_datetime: DateTime,
    estimated_cargo_weight: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Passenger {
    ssn: String,
    name: String,
    surname: String,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    birth_date: DateTime,
}

impl From<Ticket1> for Ticket {
    fn from(t: Ticket1) -> Self {
        let p = t.passenger;
        Self {
            id: t._id,
            flight_id: t.flight_id,
            passenger: Some(PassengerDetails {
                ssn: p.ssn,
                name: p.name,
                surname: p.surname,
                birth_date: convert_datetime_to_timestamp(p.birth_date),
            }),
            reservation_datetime: convert_datetime_to_timestamp(t.reservation_datetime),
            estimated_cargo_weight: t.estimated_cargo_weight,
        }
    }
}

fn convert_datetime_to_timestamp(d: DateTime) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: d.timestamp_millis() / 1000,
        nanos: (d.timestamp_millis() % 1000) as i32 * 1_000_000,
    })
}

fn convert_timestamp_to_datetime(t: Option<Timestamp>) -> Result<DateTime, Status> {
    let t = t.ok_or_else(|| Status::invalid_argument("missing timestamp"))?;
    Ok(DateTime::from_millis(
        t.seconds * 1000 + t.nanos as i64 / 1_000_000,
    ))
}

impl TryFrom<Ticket> for Ticket1 {
    type Error = Status;

    fn try_from(t: Ticket) -> Result<Self, Self::Error> {
        let Some(p) = t.passenger else {
            return Err(Status::invalid_argument("missing passenger details"));
        };

        Ok(Self {
            _id: t.id,
            flight_id: t.flight_id,
            passenger: Passenger {
                ssn: p.ssn,
                name: p.name,
                surname: p.surname,
                birth_date: convert_timestamp_to_datetime(p.birth_date)?,
            },
            reservation_datetime: convert_timestamp_to_datetime(t.reservation_datetime)?,
            estimated_cargo_weight: t.estimated_cargo_weight,
        })
    }
}

#[tonic::async_trait]
impl Tickets for TicketsApp {
    async fn list_tickets(&self, request: Request<()>) -> Result<Response<TicketList>, Status> {
        todo!()
    }
    async fn get_ticket(&self, request: Request<TicketQuery>) -> Result<Response<Ticket>, Status> {
        let TicketQuery { id } = request.into_inner();
        let collection: Collection<Ticket1> = self.mongo_client.collection("tickets");

        let Some(ticket) = collection
            .find_one(doc! { "id": &id }, None)
            .await
            .map_err(|_| Status::internal(""))?
        else {
            return Err(Status::not_found("ticket not found"));
        };

        Ok(Response::new(ticket.into()))
    }

    async fn create_ticket(&self, request: Request<Ticket>) -> Result<Response<Ticket>, Status> {
        let collection: Collection<Ticket1> = self.mongo_client.collection("tickets");

        let res = collection
            .insert_one(&request.into_inner().try_into()?, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?;

        let ticket = collection
            .find_one(doc! { "id": &res.inserted_id }, None)
            .await
            .map_err(|e| Status::from_error(Box::new(e)))?
            .ok_or_else(|| Status::internal(""))?;

        Ok(Response::new(ticket.into()))
    }

    async fn delete_ticket(&self, request: Request<TicketQuery>) -> Result<Response<()>, Status> {
        todo!()
    }
    async fn update_ticket(
        &self,
        request: Request<TicketUpdate>,
    ) -> Result<Response<Ticket>, Status> {
        todo!()
    }
}

impl TicketsApp {
    pub fn new(mongo_client: Database) -> Self {
        Self { mongo_client }
    }
}
