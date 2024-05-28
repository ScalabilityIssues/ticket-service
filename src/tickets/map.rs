use mongodb::bson::oid::ObjectId;
use tonic::Status;

use crate::datautils::{convert_datetime_to_timestamp, convert_timestamp_to_datetime};
use crate::proto::ticketsrvc::{self, TicketStatus};

use super::data;

impl From<data::Ticket> for ticketsrvc::Ticket {
    fn from(t: data::Ticket) -> Self {
        let p = t.passenger;
        let ticket_status =
            TicketStatus::from_str_name(&t.ticket_status).unwrap_or_default() as i32;

        Self {
            id: t._id.to_string(),
            flight_id: t.flight_id,
            url: t.url,
            passenger: Some(ticketsrvc::PassengerDetails {
                ssn: p.ssn,
                name: p.name,
                surname: p.surname,
                birth_date: convert_datetime_to_timestamp(p.birth_date),
                email: p.email,
            }),
            reservation_datetime: convert_datetime_to_timestamp(t.reservation_datetime),
            estimated_cargo_weight: t.estimated_cargo_weight,
            ticket_status,
        }
    }
}

impl TryFrom<ticketsrvc::Ticket> for data::Ticket {
    type Error = Status;

    fn try_from(t: ticketsrvc::Ticket) -> Result<Self, Self::Error> {
        let Some(p) = t.passenger else {
            return Err(Status::invalid_argument("missing passenger details"));
        };

        let _id = ObjectId::new();

        let ticket_status = TicketStatus::try_from(t.ticket_status)
            .or(Err(Status::invalid_argument("invalid ticket status")))?
            .as_str_name()
            .to_string();

        Ok(Self {
            _id,
            url: t.url,
            flight_id: t.flight_id,
            passenger: data::Passenger {
                ssn: p.ssn,
                name: p.name,
                surname: p.surname,
                birth_date: convert_timestamp_to_datetime(p.birth_date)?,
                email: p.email,
            },
            reservation_datetime: convert_timestamp_to_datetime(t.reservation_datetime)?,
            estimated_cargo_weight: t.estimated_cargo_weight,
            ticket_status,
        })
    }
}
