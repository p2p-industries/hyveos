use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::{stream::FuturesUnordered, Stream};

#[pin_project::pin_project]
pub struct FutureMap<K, F> {
    #[pin]
    futures: FuturesUnordered<OptionFuture<KeyFuture<K, F>>>,
    empty_waker: Option<Waker>,
}

impl<K, F> FutureMap<K, F>
where
    K: Eq,
    F: Future + Unpin,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, future: F) -> Option<F> {
        if let Some(waker) = self.empty_waker.take() {
            waker.wake();
        }

        let old = self.remove(&key);

        self.futures
            .push(OptionFuture::new(KeyFuture { key, future }));

        old
    }

    pub fn remove(&mut self, key: &K) -> Option<F> {
        self.futures
            .iter_mut()
            .find(|f| f.0.as_ref().is_some_and(|fut| fut.key == *key))?
            .take()
            .map(|fut| fut.future)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &F)> {
        self.futures
            .iter()
            .filter_map(|f| f.0.as_ref().map(|fut| (&fut.key, &fut.future)))
    }

    pub fn take_futures(&mut self) -> impl Iterator<Item = F> + '_ {
        self.futures
            .iter_mut()
            .filter_map(OptionFuture::take)
            .map(|fut| fut.future)
    }
}

impl<K, F> Stream for FutureMap<K, F>
where
    K: Clone + Eq,
    F: Future,
{
    type Item = (K, F::Output);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match futures::ready!(this.futures.as_mut().poll_next(cx)) {
                None => {
                    *this.empty_waker = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                Some(Some((key, v))) => {
                    return Poll::Ready(Some((key, v)));
                }
                Some(None) => {
                    continue;
                }
            }
        }
    }
}

impl<K, O> Default for FutureMap<K, O>
where
    K: Eq,
{
    fn default() -> Self {
        Self {
            futures: FuturesUnordered::new(),
            empty_waker: None,
        }
    }
}

#[pin_project::pin_project]
#[derive(Default)]
struct OptionFuture<F>(#[pin] Option<F>);

impl<F> OptionFuture<F> {
    fn new(fut: F) -> Self {
        Self(Some(fut))
    }

    fn take(&mut self) -> Option<F> {
        self.0.take()
    }
}

impl<F> Future for OptionFuture<F>
where
    F: Future,
{
    type Output = Option<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.project().0.as_pin_mut() {
            None => Poll::Ready(None),
            Some(fut) => fut.poll(cx).map(Some),
        }
    }
}

#[pin_project::pin_project]
struct KeyFuture<K, F> {
    key: K,
    #[pin]
    future: F,
}

impl<K, F> Future for KeyFuture<K, F>
where
    K: Clone,
    F: Future,
{
    type Output = (K, F::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let key = self.key.clone();
        let this = self.project();
        this.future.poll(cx).map(|output| (key, output))
    }
}
