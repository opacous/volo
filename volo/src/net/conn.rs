use std::net::Shutdown;
use {
    super::Address,
    async_std::{
        io::{BufReader as ReadBuf, Read as AsyncRead, Write as AsyncWrite},
        net::{TcpStream},
        os::unix::net::UnixStream,
    },
    pin_project::pin_project,
    std::{
        io::{self, IoSlice},
        pin::Pin,
        task::{Context, Poll},
    },
};

#[derive(Clone)]
pub struct ConnInfo {
    pub peer_addr: Option<Address>,
}

pub trait DynStream: AsyncRead + AsyncWrite + Send + 'static {}

impl<T> DynStream for T where T: AsyncRead + AsyncWrite + Send + 'static {}

#[pin_project(project = IoStreamProj)]
pub enum ConnStream {
    Tcp(#[pin] TcpStream),
    #[cfg(target_family = "unix")]
    Unix(#[pin] UnixStream),
}

#[pin_project(project = OwnedWriteHalfProj)]
pub enum OwnedWriteHalf {
    Tcp(#[pin] TcpStream),
    #[cfg(target_family = "unix")]
    Unix(#[pin] UnixStream),
}

impl AsyncWrite for OwnedWriteHalf {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp(half) => half.poll_write(cx, buf),
            #[cfg(target_family = "unix")]
            OwnedWriteHalfProj::Unix(half) => half.poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp(half) => half.poll_write_vectored(cx, bufs),
            #[cfg(target_family = "unix")]
            OwnedWriteHalfProj::Unix(half) => half.poll_write_vectored(cx, bufs),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp(half) => half.poll_flush(cx),
            #[cfg(target_family = "unix")]
            OwnedWriteHalfProj::Unix(half) => half.poll_flush(cx),
        }
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::from(match self.project() {
            OwnedWriteHalfProj::Tcp(half) => half.shutdown(Shutdown::Write),
            #[cfg(target_family = "unix")]
            OwnedWriteHalfProj::Unix(half) => half.shutdown(Shutdown::Write),
        })
    }
}

impl OwnedWriteHalf {
    #[inline]
    fn is_write_vectored(&self) -> bool {
        match self {
            Self::Tcp(half) => false,
            #[cfg(target_family = "unix")]
            Self::Unix(half) => false,
        }
    }
}

#[pin_project(project = OwnedReadHalfProj)]
pub enum OwnedReadHalf {
    Tcp(#[pin] TcpStream),
    #[cfg(target_family = "unix")]
    Unix(#[pin] UnixStream),
}

impl AsyncRead for OwnedReadHalf {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            OwnedReadHalfProj::Tcp(half) => half.poll_read(cx, buf),
            #[cfg(target_family = "unix")]
            OwnedReadHalfProj::Unix(half) => half.poll_read(cx, buf),
        }
    }
}

impl ConnStream {
    #[allow(clippy::type_complexity)]
    pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        match self {
            Self::Tcp(stream) => {
                let rh = stream.clone();
                let wh = stream.clone();
                (OwnedReadHalf::Tcp(rh), OwnedWriteHalf::Tcp(wh))
            }
            #[cfg(target_family = "unix")]
            Self::Unix(stream) => {

                // From https://book.async.rs/patterns/small-patterns
                // use async_std::{io, net::TcpStream};
                // async fn echo(stream: TcpStream) {
                //     let (reader, writer) = &mut (&stream, &stream);
                //     io::copy(reader, writer).await;
                // }

                let rh = stream.clone();
                let wh = stream.clone();
                (OwnedReadHalf::Unix(rh), OwnedWriteHalf::Unix(wh))
            }
        }
    }
}

impl From<TcpStream> for ConnStream {
    #[inline]
    fn from(s: TcpStream) -> Self {
        let _ = s.set_nodelay(true);
        Self::Tcp(s)
    }
}

#[cfg(target_family = "unix")]
impl From<UnixStream> for ConnStream {
    #[inline]
    fn from(s: UnixStream) -> Self {
        Self::Unix(s)
    }
}

impl AsyncRead for ConnStream {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            IoStreamProj::Tcp(s) => s.poll_read(cx, buf),
            #[cfg(target_family = "unix")]
            IoStreamProj::Unix(s) => s.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ConnStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            IoStreamProj::Tcp(s) => s.poll_write(cx, buf),
            #[cfg(target_family = "unix")]
            IoStreamProj::Unix(s) => s.poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            IoStreamProj::Tcp(s) => s.poll_flush(cx),
            #[cfg(target_family = "unix")]
            IoStreamProj::Unix(s) => s.poll_flush(cx),
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            IoStreamProj::Tcp(s) => s.poll_write_vectored(cx, bufs),
            #[cfg(target_family = "unix")]
            IoStreamProj::Unix(s) => s.poll_write_vectored(cx, bufs),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::from(match self.project() {
            IoStreamProj::Tcp(s) => s.shutdown(Shutdown::Read),
            #[cfg(target_family = "unix")]
            IoStreamProj::Unix(s) => s.shutdown(Shutdown::Read),
        })
    }
}

impl ConnStream {
    #[inline]
    fn is_write_vectored(&self) -> bool {
        match self {
            Self::Tcp(s) => true,
            #[cfg(target_family = "unix")]
            Self::Unix(s) => true,
        }
    }

    #[inline]
    pub fn peer_addr(&self) -> Option<Address> {
        match self {
            Self::Tcp(s) => s.peer_addr().map(Address::from).ok(),
            #[cfg(target_family = "unix")]
            Self::Unix(s) => s.peer_addr().ok().and_then(|s| Address::try_from(s).ok()),
        }
    }
}
pub struct Conn {
    pub stream: ConnStream,
    pub info: ConnInfo,
}

impl Conn {
    #[inline]
    pub fn new(stream: ConnStream, info: ConnInfo) -> Self {
        Conn { stream, info }
    }
}

impl<T> From<T> for Conn
where
    T: Into<ConnStream>,
{
    #[inline]
    fn from(i: T) -> Self {
        let i = i.into();
        let peer_addr = i.peer_addr();
        Conn::new(i, ConnInfo { peer_addr })
    }
}

impl AsyncRead for Conn {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Conn {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write_vectored(cx, bufs)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_close(cx)
    }
}

impl Conn {
    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }
}
