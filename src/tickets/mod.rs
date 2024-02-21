use mongodb::Database;
use tonic::{Request, Response, Status};

use crate::datautils::convert_str_to_object_id;
use crate::parse::parse_update_paths;
use crate::proto::ticketmngr::{
    tickets_server::Tickets, CreateTicketRequest, DeleteTicketRequest, GetTicketRequest, Ticket,
    TicketList, UpdateTicketRequest,
};

use self::data::TicketDatabase;

mod data;
mod map;

#[derive(Debug)]
pub struct TicketsApp {
    mongo: Database,
}

#[tonic::async_trait]
impl Tickets for TicketsApp {
    async fn list_tickets(&self, _request: Request<()>) -> Result<Response<TicketList>, Status> {
        let result = self.mongo.list_tickets().await?;

        let tickets: Vec<Ticket> = result.into_iter().map(Into::into).collect();

        Ok(Response::new(TicketList { tickets }))
    }

    async fn get_ticket(
        &self,
        request: Request<GetTicketRequest>,
    ) -> Result<Response<Ticket>, Status> {
        let GetTicketRequest { id } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;

        let ticket = self.mongo.get_ticket(id).await?;

        Ok(Response::new(ticket.into()))
    }

    async fn create_ticket(
        &self,
        request: Request<CreateTicketRequest>,
    ) -> Result<Response<Ticket>, Status> {
        let new_ticket = request.into_inner().ticket.unwrap_or_default().try_into()?;

        let id = self.mongo.create_ticket(new_ticket).await?;

        let ticket = self.mongo.get_ticket(id).await?;

        Ok(Response::new(ticket.into()))
    }

    async fn delete_ticket(
        &self,
        request: Request<DeleteTicketRequest>,
    ) -> Result<Response<()>, Status> {
        let DeleteTicketRequest { id } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;

        self.mongo.delete_ticket(id).await?;

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

        let ticket = self.mongo.get_ticket(id).await?;

        Ok(Response::new(ticket.into()))
    }
}

impl TicketsApp {
    pub fn new(mongo_client: Database) -> Self {
        Self {
            mongo: mongo_client,
        }
    }
}
