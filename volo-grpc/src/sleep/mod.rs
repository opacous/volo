use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use async_std::future;

use pin_project_lite::pin_project;

use async_std::task::{Context, Poll};

/// Stolen Straight from async_io
fn timer_after(dur: std::time::Duration) -> Timer {
    Timer::after(dur)
}

pub type Timer = async_io::Timer;


///
/// Awaits a future or times out after a duration of time.
///
/// If you want to await an I/O future consider using
/// [`io::timeout`](../io/fn.timeout.html) instead.
///
/// # Examples
///
/// ```
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// #
/// use std::time::Duration;
///
/// use async_std::future;
///
/// let never = future::pending::<()>();
/// let dur = Duration::from_millis(5);
/// assert!(future::timeout(dur, never).await.is_err());
/// #
/// # Ok(()) }) }
/// ```
pub async fn timeout(dur: Duration) -> Result<(), TimeoutError>
{
    TimeoutFuture::new(dur).await
}

pub fn make_timeout_future(dur: Duration) -> TimeoutFuture {
    TimeoutFuture::new(dur)
}

pin_project! {
    /// A future that times out after a duration of time.
    pub struct TimeoutFuture {
        #[pin]
        future: box<dyn Future<Output = ()>>,
        #[pin]
        delay: Timer,
    }
}

impl TimeoutFuture {
    #[allow(dead_code)]
    pub(super) fn new(dur: Duration) -> TimeoutFuture {
        TimeoutFuture {
            future: future::pending(),
            delay: timer_after(dur),
        }
    }
}

impl Future for TimeoutFuture {
    type Output = Result<(), TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(v) => Poll::Ready(Ok(v)),
            Poll::Pending => match this.delay.poll(cx) {
                Poll::Ready(_) => Poll::Ready(Err(TimeoutError { _private: () })),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

/// An error returned when a future times out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutError {
    _private: (),
}

impl Error for TimeoutError {}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "future has timed out".fmt(f)
    }
}
