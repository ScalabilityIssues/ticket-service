use std::str::FromStr;

use mongodb::bson::{oid::ObjectId, DateTime};
use prost_types::Timestamp;
use tonic::Status;

pub fn convert_datetime_to_timestamp(d: DateTime) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: d.timestamp_millis() / 1000,
        nanos: (d.timestamp_millis() % 1000) as i32 * 1_000_000,
    })
}

pub fn convert_timestamp_to_datetime(t: Option<Timestamp>) -> Result<DateTime, Status> {
    let t = t.ok_or_else(|| Status::invalid_argument("missing timestamp"))?;
    Ok(DateTime::from_millis(
        t.seconds * 1000 + t.nanos as i64 / 1_000_000,
    ))
}

pub fn convert_str_to_object_id(id: &str, message: &'static str) -> Result<ObjectId, Status> {
    ObjectId::from_str(id).map_err(|_| Status::invalid_argument(message))
}
