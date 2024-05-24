use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    pin::Pin,
    task::{ready, Context, Poll, Waker},
};

use futures::Stream;

#[derive(Debug)]
pub struct StreamMap<K, S> {
    streams: Vec<S>,
    map: HashMap<K, usize>,
    inverse_map: HashMap<usize, K>,
    waker: Option<Waker>,
}

impl<K: Hash + Eq + Clone, S: Stream> Default for StreamMap<K, S> {
    fn default() -> Self {
        Self {
            streams: Vec::new(),
            map: HashMap::new(),
            inverse_map: HashMap::new(),
            waker: None,
        }
    }
}

impl<K: Hash + Eq + Clone, S: Stream> StreamMap<K, S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: K, stream: S) -> Option<S> {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }

        match self.map.entry(key.clone()) {
            Entry::Occupied(entry) => {
                let index = *entry.get();
                let old_stream = std::mem::replace(&mut self.streams[index], stream);
                Some(old_stream)
            }
            Entry::Vacant(entry) => {
                let index = self.streams.len();
                self.streams.push(stream);
                self.inverse_map.insert(index, key.clone());
                entry.insert(index);
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<S>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        let index = self.map.remove(key)?;
        let stream = self.streams.swap_remove(index);

        let last_index = self.streams.len();

        if index == last_index {
            self.inverse_map.remove(&last_index);
        } else {
            let last_key = self.inverse_map.remove(&last_index).unwrap();
            self.inverse_map.insert(index, last_key.clone());
            self.map.insert(last_key, index);
        }

        Some(stream)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.contains_key(key)
    }
}

impl<K: Unpin + Hash + Eq + Clone, S: Stream + Unpin> StreamMap<K, S> {
    fn poll_next_entry(&mut self, cx: &mut Context<'_>) -> Poll<Option<(usize, S::Item)>> {
        self.waker = Some(cx.waker().clone());

        let mut order = (0..self.streams.len()).collect::<Vec<_>>();
        fastrand::shuffle(&mut order);

        let mut indices_to_remove = Vec::new();

        for i in order {
            let stream = &mut self.streams[i];

            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    return Poll::Ready(Some((i, item)));
                }
                Poll::Ready(None) => {
                    indices_to_remove.push(i);
                }
                Poll::Pending => {}
            }
        }

        for i in indices_to_remove {
            self.remove(&self.inverse_map[&i].clone());
        }

        if self.streams.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

impl<K: Unpin + Hash + Eq + Clone, S: Stream + Unpin> Stream for StreamMap<K, S> {
    type Item = (K, S::Item);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(ready!(self.poll_next_entry(cx)).map(|(i, item)| {
            let key = self.inverse_map[&i].clone();
            (key, item)
        }))
    }
}

// impl GossipSubServer {
//     pub fn new(client: Client) -> Self {
//         let (command_sender, command_receiver) = mpsc::channel(5);

//         let inner_client = client.clone();

//         tokio::spawn(async move {
//             let mut command_receiver = command_receiver;
//             let mut receivers: StreamMap<Arc<str>, _> = StreamMap::new();

//             let (message_sender, _) = broadcast::channel(5);

//             loop {
//                 tokio::select! {
//                     Some(command) = command_receiver.recv() => {
//                         match command {
//                             Command::Subscribe(topic, sender) => {
//                                 if receivers.contains_key(&topic) {
//                                     let _ = sender.send(Ok(()));
//                                     continue;
//                                 }

//                                 let receiver = match inner_client
//                                     .gossipsub()
//                                     .get_topic(IdentTopic::new(topic.to_string()))
//                                     .subscribe()
//                                     .await {
//                                     Ok(receiver) => receiver,
//                                     Err(e) => {
//                                         let _ = sender.send(Err(e));
//                                         continue;
//                                     }
//                                 };

//                                 receivers.insert(topic, BroadcastStream::new(receiver));

//                                 let _ = sender.send(Ok(()));
//                             }
//                             Command::Unsubscribe(topic) => {
//                                 receivers.remove(&topic);
//                             }
//                             Command::Recv(sender) => {
//                                 let _ = sender.send(message_sender.subscribe());
//                             }
//                         }
//                     }
//                     Some((topic, res)) = receivers.next() => {
//                         let _ = message_sender.send(res.map(move |message| (topic, message)));
//                     }
//                 }
//             }
//         });

//         Self {
//             client,
//             command_sender,
//         }
//     }
// }
