use mongodb::Database;
use tonic::{Request, Response, Status};

use crate::datautils::convert_str_to_object_id;
use crate::dependencies::FlightManager;
use crate::parse::parse_update_paths;
use crate::proto::flightmngr::Plane;
use crate::proto::ticketmngr::{
    tickets_server::Tickets, CreateTicketRequest, DeleteTicketRequest, GetTicketRequest, Ticket,
    TicketList, UpdateTicketRequest,
};
use crate::proto::ticketmngr::{
    FlightStatistics, GetFlightStatisticsRequest, ListTicketsRequest, TicketStatus,
};
use crate::rabbitmq::{Rabbit, UpdateKind};

use self::data::TicketDatabase;

mod data;
mod map;

pub struct TicketsApp {
    mongo: Database,
    flightmngr: FlightManager,
    rabbitmq: Rabbit,
}

#[tonic::async_trait]
impl Tickets for TicketsApp {
    async fn list_tickets(
        &self,
        request: Request<ListTicketsRequest>,
    ) -> Result<Response<TicketList>, Status> {
        let ListTicketsRequest { include_nonvalid } = request.into_inner();

        let result = self.mongo.list_tickets(include_nonvalid).await?;

        let tickets: Vec<Ticket> = result.into_iter().map(Into::into).collect();

        Ok(Response::new(TicketList { tickets }))
    }

    async fn get_ticket(
        &self,
        request: Request<GetTicketRequest>,
    ) -> Result<Response<Ticket>, Status> {
        let GetTicketRequest { id, allow_nonvalid } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;

        let ticket = self.mongo.get_ticket(id, allow_nonvalid).await?;

        Ok(Response::new(ticket.into()))
    }

    async fn create_ticket(
        &self,
        request: Request<CreateTicketRequest>,
    ) -> Result<Response<Ticket>, Status> {
        let mut new_ticket = request.into_inner().ticket.unwrap_or_default();
        new_ticket.ticket_status = Into::into(TicketStatus::Valid);

        let existing_tickets = self
            .mongo
            .get_existing_tickets(&new_ticket.flight_id)
            .await?;
        let Plane { cabin_capacity, .. } = self
            .flightmngr
            .get_plane_details(new_ticket.flight_id.clone())
            .await?;

        // TODO: prevent race condition when creating a ticket
        if cabin_capacity - existing_tickets <= 0 {
            return Err(Status::failed_precondition("no seat available"));
        }

        let id = self.mongo.create_ticket(new_ticket.try_into()?).await?;

        let ticket: Ticket = self.mongo.get_ticket(id, false).await?.into();

        self.rabbitmq
            .notify_ticket_update(ticket.clone(), UpdateKind::Create)
            .await?;

        Ok(Response::new(ticket))
    }

    async fn delete_ticket(
        &self,
        request: Request<DeleteTicketRequest>,
    ) -> Result<Response<()>, Status> {
        let DeleteTicketRequest { id } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;

        let ticket = self.mongo.get_ticket(id, false).await?.into();

        self.mongo.delete_ticket(id).await?;

        self.rabbitmq
            .notify_ticket_update(ticket, UpdateKind::Delete)
            .await?;

        Ok(Response::new(()))
    }

    async fn update_ticket(
        &self,
        request: Request<UpdateTicketRequest>,
    ) -> Result<Response<Ticket>, Status> {
        let UpdateTicketRequest {
            id,
            update,
            update_mask,
        } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;
        let update_paths = parse_update_paths(update_mask)?;
        let update = update.ok_or(Status::invalid_argument("update required"))?;

        self.mongo
            .update_ticket(id, update.try_into()?, update_paths)
            .await?;

        let ticket: Ticket = self.mongo.get_ticket(id, false).await?.into();

        self.rabbitmq
            .notify_ticket_update(ticket.clone(), UpdateKind::Update)
            .await?;

        Ok(Response::new(ticket))
    }

    async fn get_flight_statistics(
        &self,
        request: Request<GetFlightStatisticsRequest>,
    ) -> Result<Response<FlightStatistics>, Status> {
        let GetFlightStatisticsRequest { flight_id } = request.into_inner();
        // let id = convert_str_to_object_id(&flight_id, "invalid id")?;

        let existing_tickets = self.mongo.get_existing_tickets(&flight_id).await?;
        let airplane = self.flightmngr.get_plane_details(flight_id).await?;

        Ok(Response::new(FlightStatistics {
            total_seats: airplane.cabin_capacity,
            reserved_seats: existing_tickets,
        }))
    }
}

impl TicketsApp {
    pub fn new(mongo_client: Database, flightmngr: FlightManager, rabbitmq: Rabbit) -> Self {
        Self {
            mongo: mongo_client,
            flightmngr,
            rabbitmq,
        }
    }
}
