use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use derivative::Derivative;
use futures::Future;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

// this shit is funny, we need to make sure this is send so we can use it in tokio
type SnifferFunction = Box<dyn Fn(&Vec<u8>) + Send>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SnifferBuffer {
    read_done: bool,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,
    #[derivative(Debug = "ignore")]
    callback: SnifferFunction,
}

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

impl Default for SnifferBuffer {
    fn default() -> Self {
        Self {
            read_done: false,
            pos: 0,
            cap: 0,
            amt: 0,
            buf: vec![0; 65536].into_boxed_slice(),
            callback: Box::new(|_: &Vec<u8>| {}),
            // _marker: std::marker::PhantomData::<'a>::new(),
        }
    }
}

impl SnifferBuffer {
    pub fn set_callback(&mut self, cb: SnifferFunction) {
        self.callback = cb
    }

    pub fn poll_copy<R, W>(
        &mut self,
        cx: &mut Context<'_>,
        mut reader: Pin<&mut R>,
        mut writer: Pin<&mut W>,
    ) -> Poll<io::Result<u64>>
    where
        R: AsyncRead + ?Sized,
        W: AsyncWrite + ?Sized,
    {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let me = &mut *self;
                let mut buf = ReadBuf::new(&mut me.buf);
                ready!(reader.as_mut().poll_read(cx, &mut buf))?;
                let n = buf.filled().len();
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            // If our buffer has some data, let's write it out!
            while self.pos < self.cap {
                let me = &mut *self;

                // let i = ready!(self.poll_write_buf(cx, reader.as_mut(), writer.as_mut()))?;
                // if i == 0 {
                //     return Poll::Ready(Err(io::Error::new(
                //         io::ErrorKind::WriteZero,
                //         "write zero byte into writer",
                //     )));
                // } else {
                //     self.pos += i;
                //     self.amt += i as u64;
                //     self.need_flush = true;
                // }
                let buffer = Vec::from(&me.buf[me.pos..me.cap]);
                (self.callback)(&buffer);

                let i = ready!(writer.as_mut().poll_write(cx, &buffer))?;
                if i == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "write zero byte into writer",
                    )));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                }
            }

            // If pos larger than cap, this loop will never stop.
            // In particular, user's wrong poll_write implementation returning
            // incorrect written length may lead to thread blocking.
            debug_assert!(
                self.pos <= self.cap,
                "writer returned length larger than input slice"
            );

            // If we've written all the data and we've seen EOF, flush out the
            // data and finish the transfer.
            if self.pos == self.cap && self.read_done {
                ready!(writer.as_mut().poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}

/// A future that asynchronously copies the entire contents of a reader into a
/// writer.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct Copy<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    writer: &'a mut W,
    buf: SnifferBuffer,
}
pub async fn copy<'a, R, W>(
    reader: &'a mut R,
    writer: &'a mut W,
    buf: SnifferBuffer,
) -> io::Result<u64>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    Copy {
        reader,
        writer,
        buf,
    }
    .await
}

impl<R, W> Future for Copy<'_, R, W>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<u64>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        let me = &mut *self;

        me.buf
            .poll_copy(cx, Pin::new(&mut *me.reader), Pin::new(&mut *me.writer))
    }
}
