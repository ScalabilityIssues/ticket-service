use mongodb::bson::oid::ObjectId;
use tonic::Status;

use crate::datautils::convert_str_to_object_id;
use crate::datautils::{convert_datetime_to_timestamp, convert_timestamp_to_datetime};
use crate::proto::ticketmngr;

use super::data;

impl From<data::Ticket> for ticketmngr::Ticket {
    fn from(t: data::Ticket) -> Self {
        let p = t.passenger;
        Self {
            id: t._id.to_string(),
            flight_id: t.flight_id,
            passenger: Some(ticketmngr::PassengerDetails {
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

impl TryFrom<ticketmngr::Ticket> for data::Ticket {
    type Error = Status;

    fn try_from(t: ticketmngr::Ticket) -> Result<Self, Self::Error> {
        let Some(p) = t.passenger else {
            return Err(Status::invalid_argument("missing passenger details"));
        };

        let _id = match t.id.as_str() {
            "" => ObjectId::new(),
            id => convert_str_to_object_id(id, "invalid id")?,
        };

        Ok(Self {
            _id,
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
