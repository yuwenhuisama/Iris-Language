//! `IRIS-V1-FFI-C012` thread affinity and the `C014` post queue.

use crate::IrisStatus;
use std::sync::{Arc, Mutex};

/// Guards Iris state against off-thread access.
///
/// `IRIS-V1-FFI-C012` requires every ABI call touching handles, the managed
/// heap, metadata, dispatch, the scheduler or `ExceptionContext` graphs to run
/// on the OWNING runtime thread, and to answer a thread-affinity status rather
/// than racing the heap when it does not.
#[derive(Debug)]
pub struct ThreadAffinity {
    owner: std::thread::ThreadId,
}

impl ThreadAffinity {
    /// Binds the runtime to the calling thread.
    #[must_use]
    pub fn bind_current() -> Self {
        Self {
            owner: std::thread::current().id(),
        }
    }

    /// Checks the caller may touch Iris state.
    ///
    /// This is the gate `C016` marks Prohibited for a worker thread, so it
    /// answers a status rather than panicking or proceeding.
    pub fn check(&self) -> IrisStatus {
        if std::thread::current().id() == self.owner {
            IrisStatus::Success
        } else {
            IrisStatus::ThreadAffinity
        }
    }
}

/// One completion posted from an external thread.
///
/// `IRIS-V1-FFI-C013` lets an external thread post COPIED or externally owned
/// data only, never a handle, because the runtime thread is what converts
/// posted data into Iris values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Post {
    /// The token authorizing exactly one completion.
    pub token: u64,
    /// Copied payload bytes; no Iris handle may appear here.
    pub payload: Vec<u8>,
}

/// The thread-safe post queue.
///
/// `IRIS-V1-FFI-C014` makes this the ONLY v1 cross-thread entry into a runtime
/// and requires FIFO ordering for posts accepted from ONE producer in
/// submission order. Ordering between independent producers is not specified,
/// which is why acceptance order is the only guarantee recorded here.
#[derive(Clone, Debug)]
pub struct PostQueue {
    inner: Arc<Mutex<QueueState>>,
}

#[derive(Debug, Default)]
struct QueueState {
    pending: Vec<Post>,
    /// Tokens already used, so a second post is refused.
    ///
    /// `IRIS-V1-FFI-C037` makes the FIRST completion stand.
    spent: Vec<u64>,
}

impl Default for PostQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PostQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(QueueState::default())),
        }
    }

    /// Posts one completion from any thread.
    ///
    /// A token authorizes exactly one completion, so a repeat answers
    /// `DuplicateCompletion` and the first post stands.
    pub fn post(&self, post: Post) -> IrisStatus {
        let Ok(mut state) = self.inner.lock() else {
            return IrisStatus::InvalidRuntime;
        };
        if state.spent.contains(&post.token)
            || state.pending.iter().any(|held| held.token == post.token)
        {
            return IrisStatus::DuplicateCompletion;
        }
        state.pending.push(post);
        IrisStatus::Success
    }

    /// Discards every pending post and spent token.
    ///
    /// `IRIS-V1-FFI-C014` makes this queue process-wide, so one scenario's
    /// accepted post outlives it and the next scenario would observe a shared
    /// total. Resetting a runtime therefore has to clear the queue too, or the
    /// reset is not a reset.
    pub fn clear(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.pending.clear();
            state.spent.clear();
        }
    }

    /// Drains accepted posts in submission order, on the runtime thread.
    pub fn drain(&self, affinity: &ThreadAffinity) -> Result<Vec<Post>, IrisStatus> {
        // C013 makes the RUNTIME thread the one that converts posted data into
        // Iris values, so draining is itself an owning-thread operation.
        if affinity.check() != IrisStatus::Success {
            return Err(IrisStatus::ThreadAffinity);
        }
        let Ok(mut state) = self.inner.lock() else {
            return Err(IrisStatus::InvalidRuntime);
        };
        let drained = std::mem::take(&mut state.pending);
        for post in &drained {
            state.spent.push(post.token);
        }
        Ok(drained)
    }
}

#[cfg(test)]
mod tests {
    use super::{Post, PostQueue, ThreadAffinity};
    use crate::IrisStatus;

    #[test]
    fn c012_the_owning_thread_may_touch_iris_state() {
        // Given
        let affinity = ThreadAffinity::bind_current();

        // When / Then
        assert_eq!(affinity.check(), IrisStatus::Success);
    }

    #[test]
    fn c012_a_worker_thread_gets_a_thread_affinity_status() {
        // Given
        let affinity = ThreadAffinity::bind_current();

        // When the check runs on another thread, C016 marks the operation
        // Prohibited, so it must report rather than race the heap.
        let observed = std::thread::scope(|scope| {
            let Ok(observed) = scope.spawn(|| affinity.check()).join() else {
                unreachable!("the worker returns a status")
            };
            observed
        });

        // Then
        assert_eq!(observed, IrisStatus::ThreadAffinity);
    }

    #[test]
    fn c014_posts_from_one_producer_keep_submission_order() {
        // Given
        let affinity = ThreadAffinity::bind_current();
        let queue = PostQueue::new();

        // When
        for token in 1..=3 {
            assert_eq!(
                queue.post(Post {
                    token,
                    payload: vec![token as u8],
                }),
                IrisStatus::Success
            );
        }

        // Then
        let Ok(drained) = queue.drain(&affinity) else {
            unreachable!("the owning thread drains")
        };
        assert_eq!(
            drained.iter().map(|post| post.token).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn c037_a_second_post_for_one_token_is_refused() {
        // Given
        let queue = PostQueue::new();
        assert_eq!(
            queue.post(Post {
                token: 7,
                payload: vec![9],
            }),
            IrisStatus::Success
        );

        // When / Then the first completion stands.
        assert_eq!(
            queue.post(Post {
                token: 7,
                payload: vec![9],
            }),
            IrisStatus::DuplicateCompletion
        );
    }

    #[test]
    fn c037_a_token_stays_spent_after_draining() {
        // Given
        let affinity = ThreadAffinity::bind_current();
        let queue = PostQueue::new();
        queue.post(Post {
            token: 7,
            payload: vec![9],
        });
        let _ = queue.drain(&affinity);

        // When / Then reusing the token after delivery is still refused.
        assert_eq!(
            queue.post(Post {
                token: 7,
                payload: vec![9],
            }),
            IrisStatus::DuplicateCompletion
        );
    }

    #[test]
    fn c013_an_external_thread_may_post_but_not_drain() {
        // Given
        let affinity = ThreadAffinity::bind_current();
        let queue = PostQueue::new();

        // When a worker posts copied data, that is permitted; draining is not.
        let observed = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    (
                        queue.post(Post {
                            token: 1,
                            payload: vec![5],
                        }),
                        queue.drain(&affinity).err(),
                    )
                })
                .join()
        });
        let Ok((posted, drained)) = observed else {
            unreachable!("the worker returns both statuses")
        };

        // Then
        assert_eq!(posted, IrisStatus::Success);
        assert_eq!(drained, Some(IrisStatus::ThreadAffinity));
    }
}
