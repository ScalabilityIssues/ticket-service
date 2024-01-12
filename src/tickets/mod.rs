use mongodb::Database;
use tonic::{Request, Response, Status};

use crate::datautils::convert_str_to_object_id;
use crate::proto::ticketmngr::{
    tickets_server::Tickets, Ticket, TicketList, TicketQuery, TicketUpdate,
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

    async fn get_ticket(&self, request: Request<TicketQuery>) -> Result<Response<Ticket>, Status> {
        let TicketQuery { id } = request.into_inner();
        let id = convert_str_to_object_id(&id, "invalid id")?;

        let ticket = self.mongo.get_ticket(id).await?;

        Ok(Response::new(ticket.into()))
    }

    async fn create_ticket(&self, request: Request<Ticket>) -> Result<Response<Ticket>, Status> {
        let new_ticket = request.into_inner().try_into()?;

        let id = self.mongo.create_ticket(new_ticket).await?;

        let ticket = self.mongo.get_ticket(id).await?;

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
        Self {
            mongo: mongo_client,
        }
    }
}
