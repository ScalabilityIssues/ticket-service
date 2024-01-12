use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime;
use mongodb::Database;
use prost_types::Timestamp;
use tonic::{Request, Response, Status};

use crate::proto::ticketmngr::PassengerDetails;
use crate::proto::ticketmngr::{
    tickets_server::Tickets, Ticket, TicketList, TicketQuery, TicketUpdate,
};

use self::data::TicketDatabase;

mod data;

#[derive(Debug)]
pub struct TicketsApp {
    mongo_client: Database,
}

impl From<data::Ticket> for Ticket {
    fn from(t: data::Ticket) -> Self {
        let p = t.passenger;
        Self {
            id: t._id.to_string(),
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

impl TryFrom<Ticket> for data::Ticket {
    type Error = Status;

    fn try_from(t: Ticket) -> Result<Self, Self::Error> {
        let Some(p) = t.passenger else {
            return Err(Status::invalid_argument("missing passenger details"));
        };

        Ok(Self {
            _id: ObjectId::from_str(&t.id).map_err(|_| Status::invalid_argument("invalid id"))?,
            flight_id: t.flight_id,
            passenger: data::Passenger {
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
    async fn list_tickets(&self, _request: Request<()>) -> Result<Response<TicketList>, Status> {
        let result = self.mongo_client.list_tickets().await?;

        let tickets: Vec<Ticket> = result.into_iter().map(|t| t.into()).collect();

        Ok(Response::new(TicketList { tickets }))
    }

    async fn get_ticket(&self, request: Request<TicketQuery>) -> Result<Response<Ticket>, Status> {
        let TicketQuery { id } = request.into_inner();
        let id = ObjectId::from_str(&id).map_err(|_| Status::invalid_argument("invalid id"))?;

        let ticket = self.mongo_client.get_ticket(id).await?;

        Ok(Response::new(ticket.into()))
    }

    async fn create_ticket(&self, request: Request<Ticket>) -> Result<Response<Ticket>, Status> {
        let id = self
            .mongo_client
            .create_ticket(request.into_inner().try_into()?)
            .await?;

        let ticket = self.mongo_client.get_ticket(id).await?;

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
