use std::thread;
use std::time::Duration;
use time::OffsetDateTime;

use crate::id::Id;

/// Generator for IDs with a specific worker ID.
///
/// This generator creates IDs that include a worker identifier,
/// making them suitable for distributed systems where each worker
/// needs to generate unique IDs.
#[derive(Debug, Clone, Copy)]
pub struct WorkerIdGenerator {
    worker: u16,
    next: u16,
}

/// Duration to sleep after overflowing the sequence number.
/// Used to avoid collisions.
const SLEEP_DURATION: Duration = Duration::from_secs(1);

impl WorkerIdGenerator {
    /// Creates a new worker ID generator.
    #[must_use]
    pub const fn new(worker: u16) -> Self {
        Self { next: 0, worker }
    }

    /// Generates a new random id.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bomboni_common::id::worker::WorkerIdGenerator;
    ///
    /// let mut g = WorkerIdGenerator::new(1);
    /// assert_ne!(g.generate(), g.generate());
    /// ```
    pub fn generate(&mut self) -> Id {
        let id = Id::from_worker_parts(OffsetDateTime::now_utc(), self.worker, self.next);

        self.next += 1;
        if self.next == u16::MAX {
            self.next = 0;
            thread::sleep(SLEEP_DURATION);
        }

        id
    }

    /// Generates a new random id.
    ///
    /// The same as [`Self::generate`] but async.
    #[cfg(feature = "tokio")]
    pub async fn generate_async(&mut self) -> Id {
        let id = Id::from_worker_parts(OffsetDateTime::now_utc(), self.worker, self.next);

        self.next += 1;
        if self.next == u16::MAX {
            self.next = 0;
            tokio::time::sleep(SLEEP_DURATION).await;
        }

        id
    }

    /// Generates a series of random ids.
    /// Faster than [`Self::generate`] for multiple ids at a time.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use bomboni_common::id::worker::WorkerIdGenerator;
    ///
    /// let mut g = WorkerIdGenerator::new(1);
    /// let ids = g.generate_multiple(3);
    /// let id_set: HashSet<_> = ids.iter().collect();
    /// assert_eq!(id_set.len(), ids.len());
    /// ```
    pub fn generate_multiple(&mut self, count: usize) -> Vec<Id> {
        if count == 0 {
            return Vec::new();
        }

        let mut ids = Vec::with_capacity(count);
        let mut now = OffsetDateTime::now_utc();

        for _ in 0..count {
            let id = Id::from_worker_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                thread::sleep(SLEEP_DURATION);
                now = OffsetDateTime::now_utc();
            }
        }

        ids
    }

    /// Generates a series of random ids in async context.
    ///
    /// The same as [`Self::generate_multiple`] but async.
    #[cfg(feature = "tokio")]
    pub async fn generate_multiple_async(&mut self, count: usize) -> Vec<Id> {
        if count == 0 {
            return Vec::new();
        }

        let mut ids = Vec::with_capacity(count);
        let mut now = OffsetDateTime::now_utc();

        for _ in 0..count {
            let id = Id::from_worker_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                tokio::time::sleep(SLEEP_DURATION).await;
                now = OffsetDateTime::now_utc();
            }
        }

        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut id_generator = WorkerIdGenerator::new(42);
        let id = id_generator.generate();
        let (_timestamp, worker, sequence) = id.decode_worker();
        assert_eq!(worker, 42);
        let id = id_generator.generate();
        assert_ne!(sequence, id.decode_worker().2);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn generate_multiple() {
        use std::collections::HashSet;
        const N: usize = 10;

        let mut g = WorkerIdGenerator::new(1);

        let mut ids = HashSet::new();
        ids.extend(g.generate_multiple_async(N / 2).await);
        g.next = u16::MAX - 1;
        ids.extend(g.generate_multiple_async(N / 2).await);

        assert_eq!(ids.len(), N);
        let id_set: HashSet<_> = ids.iter().collect();
        assert_eq!(id_set.len(), N);
    }
}
