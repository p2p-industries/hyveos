use std::{io, net::SocketAddr, task::{Context, Poll}};
use tokio::{io::ReadBuf, net::UdpSocket as TokioUdpSocket};

pub trait AsyncSocket: Unpin + Send + 'static {
    fn poll_read(
        &mut self,
        _cx: &mut Context,
        _buf: &mut [u8],
    ) -> Poll<Result<(usize, SocketAddr), io::Error>>;

    fn poll_write(
        &mut self,
        _cx: &mut Context,
        _packet: &[u8],
        _to: SocketAddr,
    ) -> Poll<Result<(), io::Error>>;
}

impl AsyncSocket for TokioUdpSocket {
    fn poll_read(
        &mut self,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<(usize, SocketAddr), io::Error>> {
        let mut rbuf = ReadBuf::new(buf);
        match self.poll_recv_from(cx, &mut rbuf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Ready(Ok(addr)) => Poll::Ready(Ok((rbuf.filled().len(), addr))),
        }
    }

    fn poll_write(
        &mut self,
        cx: &mut Context,
        packet: &[u8],
        to: SocketAddr,
    ) -> Poll<Result<(), io::Error>> {
        match self.poll_send_to(cx, packet, to) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Ready(Ok(_len)) => Poll::Ready(Ok(())),
        }
    }
}
