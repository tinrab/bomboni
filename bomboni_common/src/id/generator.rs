use std::thread;
use std::time::Duration;

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
use wasm_bindgen::prelude::*;

#[cfg(feature = "tokio")]
use parking_lot::Mutex;
#[cfg(feature = "tokio")]
use std::{ops::Deref, sync::Arc};

use crate::date_time::UtcDateTime;
use crate::id::Id;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    all(
        target_family = "wasm",
        not(any(target_os = "emscripten", target_os = "wasi")),
        feature = "wasm"
    ),
    wasm_bindgen(js_name = IdGenerator)
)]
pub struct IdGenerator {
    worker: u16,
    next: u16,
}

#[cfg(feature = "tokio")]
#[derive(Debug, Clone)]
pub struct IdGeneratorArc(Arc<Mutex<IdGenerator>>);

/// Duration to sleep after overflowing the sequence number.
/// Used to avoid collisions.
const SLEEP_DURATION: Duration = Duration::from_secs(1);

impl IdGenerator {
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
    /// use bomboni_common::id::generator::IdGenerator;
    ///
    /// let mut g = IdGenerator::new(1);
    /// assert_ne!(g.generate(), g.generate());
    /// ```
    pub fn generate(&mut self) -> Id {
        let id = Id::from_parts(UtcDateTime::now(), self.worker, self.next);

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
        let id = Id::from_parts(UtcDateTime::now(), self.worker, self.next);

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
        let mut now = UtcDateTime::now();

        for _ in 0..count {
            let id = Id::from_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                thread::sleep(SLEEP_DURATION);
                now = UtcDateTime::now();
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
        let mut now = UtcDateTime::now();

        for _ in 0..count {
            let id = Id::from_parts(now, self.worker, self.next);
            ids.push(id);

            self.next += 1;
            if self.next == u16::MAX {
                self.next = 0;
                tokio::time::sleep(SLEEP_DURATION).await;
                now = UtcDateTime::now();
            }
        }

        ids
    }
}

#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm",
))]
#[wasm_bindgen(js_class = IdGenerator)]
impl IdGenerator {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(worker: u16) -> Self {
        Self { next: 0, worker }
    }

    #[wasm_bindgen(js_name = generate)]
    pub fn wasm_generate(&mut self) -> Id {
        self.generate()
    }
}

#[cfg(feature = "tokio")]
const _: () = {
    impl IdGeneratorArc {
        pub fn new(worker: u16) -> Self {
            Self(Arc::new(Mutex::new(IdGenerator::new(worker))))
        }
    }

    impl Deref for IdGeneratorArc {
        type Target = Mutex<IdGenerator>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

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
        use std::collections::HashSet;
        const N: usize = 10;

        let mut g = IdGenerator::new(1);

        let mut ids = HashSet::new();
        ids.extend(g.generate_multiple_async(N / 2).await);
        g.next = u16::MAX - 1;
        ids.extend(g.generate_multiple_async(N / 2).await);

        assert_eq!(ids.len(), N);
        let id_set: HashSet<_> = ids.iter().collect();
        assert_eq!(id_set.len(), N);
    }
}
