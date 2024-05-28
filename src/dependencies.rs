use tonic::{transport::Channel, Status};

use crate::proto::{
    flightmngr::{
        flights_client::FlightsClient, planes_client::PlanesClient, GetFlightRequest,
        GetPlaneRequest, Plane,
    },
    ticketsrvc::Ticket,
    validationsvc::{validation_client::ValidationClient, SignTicketRequest, SignTicketResponse},
};

#[derive(Debug, Clone)]
pub struct FlightManager {
    planes_client: PlanesClient<Channel>,
    flights_client: FlightsClient<Channel>,
}

impl FlightManager {
    pub fn new(channel: Channel) -> Self {
        Self {
            planes_client: PlanesClient::new(channel.clone()),
            flights_client: FlightsClient::new(channel),
        }
    }

    pub async fn get_plane_details(&self, flight_id: String) -> Result<Plane, Status> {
        let flight = self
            .flights_client
            .clone()
            .get_flight(GetFlightRequest { id: flight_id })
            .await?
            .into_inner();

        let airplane = self
            .planes_client
            .clone()
            .get_plane(GetPlaneRequest {
                id: flight.plane_id,
            })
            .await?
            .into_inner();

        Ok(airplane)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationService {
    pub validation_client: ValidationClient<Channel>,
}

impl ValidationService {
    pub fn new(channel: Channel) -> Self {
        Self {
            validation_client: ValidationClient::new(channel),
        }
    }

    pub async fn make_qr_code(&self, ticket: Ticket) -> Result<Vec<u8>, Status> {
        let SignTicketResponse { qr } = self
            .validation_client
            .clone()
            .sign_ticket(SignTicketRequest {
                ticket: Some(ticket.clone()),
            })
            .await?
            .into_inner();

        Ok(qr)
    }
}
