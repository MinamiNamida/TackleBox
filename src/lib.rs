pub mod contracts;
pub mod connection {
    tonic::include_proto!("client");
    tonic::include_proto!("sponsor");
}
