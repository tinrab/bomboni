use std::{
    thread,
    time::{Duration, SystemTime},
};

use super::Id;

#[derive(Debug, Clone, Copy)]
pub struct IdGenerator {
    worker: u16,
    next: u16,
}

/// Duration to sleep after overflowing the sequence number.
/// Used to avoid collisions.
const SLEEP_DURATION: Duration = Duration::from_secs(1);

#[cfg(feature = "tokio")]
pub type IdGeneratorArc = std::sync::Arc<tokio::sync::Mutex<IdGenerator>>;

impl IdGenerator {
    pub fn new(worker: u16) -> Self {
        IdGenerator { next: 0, worker }
    }

    /// Generates a new random id.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bomboni_common::id::generator::IdGenerator;
    ///
    /// let mut g = IdGenerator::new(1);
    /// assert_ne!(g.generate(), g.generate());
    /// ```
    pub fn generate(&mut self) -> Id {
        let id = Id::from_parts(SystemTime::now(), self.worker, self.next);

        self.next += 1;
        if self.next == u16::MAX {
            self.next = 0;
            thread::sleep(SLEEP_DURATION);
        }

        id
    }

    /// Generates a new random id.
    ///
    /// The same as [`generate`] but async.
    #[cfg(feature = "tokio")]
    pub async fn generate_async(&mut self) -> Id {
        let id = Id::from_parts(SystemTime::now(), self.worker, self.next);

        self.next += 1;
        if self.next == u16::MAX {
            self.next = 0;
            tokio::time::sleep(SLEEP_DURATION).await;
        }

        id
    }

    /// Generates a series of random ids.
    /// Faster than [`generate`] for multiple ids at a time.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::collections::HashSet;
    /// use bomboni_common::id::generator::IdGenerator;
    ///
    /// let mut g = IdGenerator::new(1);
    /// let ids = g.generate_multiple(3);
    /// let id_set: HashSet<_> = ids.iter().collect();
    /// assert_eq!(id_set.len(), ids.len());
    /// ```
    pub fn generate_multiple(&mut self, count: usize) -> Vec<Id> {
        if count == 0 {
            return Vec::new();
        }

        let mut ids = Vec::with_capacity(count);
        let mut now = SystemTime::now();

        for _ in 0..count {
            let id = Id::from_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                thread::sleep(SLEEP_DURATION);
                now = SystemTime::now();
            }
        }

        ids
    }

    /// Generates a series of random ids in async context.
    ///
    /// The same as [`generate_multiple`] but async.
    #[cfg(feature = "tokio")]
    pub async fn generate_multiple_async(&mut self, count: usize) -> Vec<Id> {
        if count == 0 {
            return Vec::new();
        }

        let mut ids = Vec::with_capacity(count);
        let mut now = SystemTime::now();

        for _ in 0..count {
            let id = Id::from_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                tokio::time::sleep(SLEEP_DURATION).await;
                now = SystemTime::now();
            }
        }

        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn it_works() {
        let mut id_generator = IdGenerator::new(42);
        let id = id_generator.generate();
        let (_timestamp, worker, sequence) = id.decode();
        assert_eq!(worker, 42);
        let id = id_generator.generate();
        assert_ne!(sequence, id.decode().2);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn generate_multiple() {
        let mut g = IdGenerator::new(1);

        const N: usize = 10;
        let mut ids = HashSet::new();
        ids.extend(g.generate_multiple_async(N / 2).await);
        g.next = u16::MAX - 1;
        ids.extend(g.generate_multiple_async(N / 2).await);

        assert_eq!(ids.len(), N);
        let id_set: HashSet<_> = ids.iter().collect();
        assert_eq!(id_set.len(), N);
    }
}
