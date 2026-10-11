use std::sync::{Mutex, PoisonError};

use time::{Duration, OffsetDateTime};

/// A source of the current time. The ledger takes it as a parameter so tests can cross midnight
/// and retention (T-07) can reuse the same seam.
pub trait Clock: Send + Sync + std::fmt::Debug {
    /// The current instant. Implementations return UTC.
    fn now(&self) -> OffsetDateTime;
}

/// The real clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// A clock that only moves when told to. For tests.
#[derive(Debug)]
pub struct ManualClock {
    now: Mutex<OffsetDateTime>,
}

impl ManualClock {
    /// A clock stopped at `start`.
    #[must_use]
    pub fn new(start: OffsetDateTime) -> Self {
        Self {
            now: Mutex::new(start),
        }
    }

    /// Jump to `instant`.
    pub fn set(&self, instant: OffsetDateTime) {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner) = instant;
    }

    /// Move forward by `by`.
    pub fn advance(&self, by: Duration) {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> OffsetDateTime {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
