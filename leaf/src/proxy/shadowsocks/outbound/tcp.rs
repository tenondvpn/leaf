use std::io;
extern crate rand;
use rand::Rng;
use async_trait::async_trait;
use bytes::BytesMut;
use tokio::io::AsyncWriteExt;

use super::shadow::ShadowedStream;
use crate::{
    proxy::*,
    session::{Session, SocksAddrWireType},
};

pub struct Handler {
    pub address: String,
    pub port: u16,
    pub cipher: String,
    pub password: String,
}

#[async_trait]
impl TcpOutboundHandler for Handler {
    type Stream = AnyStream;

    fn connect_addr(&self) -> Option<OutboundConnect> {
        let tmp_vec: Vec<&str> = self.password.split("M").collect();
        let tmp_route = tmp_vec[1].to_string();
        let route_vec: Vec<&str> = tmp_route.split("-").collect();
        let mut rng = rand::thread_rng();
        let rand_idx = rng.gen_range(0..route_vec.len());
        let ip_port = route_vec[rand_idx].to_string();
        let ip_port_vec: Vec<&str> = ip_port.split("N").collect();
        let address = ip_port_vec[0].to_string();
        let port: u16 = ip_port_vec[1].parse::<u16>().unwrap();
        Some(OutboundConnect::Proxy(address.clone(), port))    }

    async fn handle<'a>(
        &'a self,
        sess: &'a Session,
        stream: Option<Self::Stream>,
    ) -> io::Result<Self::Stream> {
        let stream = stream.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid input"))?;
        let mut stream = ShadowedStream::new(stream, &self.cipher, &self.password)?;
        let mut buf = BytesMut::new();
        sess.destination
            .write_buf(&mut buf, SocksAddrWireType::PortLast);
        stream.write_all(&buf).await?;
        Ok(Box::new(stream))
    }
}
