use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};
use tower_http::trace;
use tracing::Level;

use crate::dependencies::FlightManager;
use crate::{proto::ticketmngr::tickets_server::TicketsServer, tickets::TicketsApp};

mod config;
mod datautils;
mod dependencies;
mod errors;
mod parse;
mod proto;
mod tickets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let opt = envy::from_env::<config::Options>()?;

    // define db
    let mut client_options = ClientOptions::parse(&opt.database_url).await?;
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    let client = Client::with_options(client_options)?.database("ticket-svc");
    client.run_command(doc! { "ping": 1 }, None).await?;
    tracing::info!("succcessfully pinged database");

    // define flightmngr grpc client
    let channel = Channel::builder(opt.flightmngr_url.try_into()?)
        .connect()
        .await?;

    // bind server socket
    let addr = SocketAddr::new(opt.ip, opt.port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("starting server on {}", addr);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        // configure the server
        .timeout(std::time::Duration::from_secs(10))
        .layer(
            trace::TraceLayer::new_for_grpc()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // enable grpc reflection
        .add_service(reflection)
        .add_service(TicketsServer::new(TicketsApp::new(
            client,
            FlightManager::new(channel),
        )))
        // serve
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
            let _ = signal(SignalKind::terminate()).unwrap().recv().await;
            tracing::info!("shutting down");
        })
        .await?;

    Ok(())
}
