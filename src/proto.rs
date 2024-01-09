// tonic::include_proto!("_");

pub mod ticketmngr {
    tonic::include_proto!("ticketsrvc");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("proto_descriptor");
