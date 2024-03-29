// tonic::include_proto!("_");

pub mod ticketsrvc {
    tonic::include_proto!("ticketsrvc");
}

pub mod flightmngr {
    tonic::include_proto!("flightmngr");
}

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("proto_descriptor");
