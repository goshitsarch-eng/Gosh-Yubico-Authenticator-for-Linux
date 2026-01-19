use glib::clone;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Timer service for TOTP countdown
///
/// Provides periodic callbacks synchronized with TOTP time periods.
pub struct TimerService {
    /// TOTP period in seconds (typically 30)
    period: u32,
    /// Source ID for the main timer
    source_id: Rc<Cell<Option<glib::SourceId>>>,
}

impl std::fmt::Debug for TimerService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimerService")
            .field("period", &self.period)
            .finish()
    }
}

impl TimerService {
    /// Create a new timer service with the given period
    pub fn new(period: u32) -> Self {
        Self {
            period,
            source_id: Rc::new(Cell::new(None)),
        }
    }

    /// Start the timer with callbacks for period changes and countdown ticks
    ///
    /// - `on_period_change`: Called when a new TOTP period begins (every 30 seconds)
    /// - `on_tick`: Called every second with the remaining seconds in the current period
    pub fn start<F, G>(&self, on_period_change: F, on_tick: G)
    where
        F: Fn() + 'static,
        G: Fn(u32, f64) + 'static,
    {
        let period = self.period;
        let on_period_change = Rc::new(on_period_change);
        let on_tick = Rc::new(on_tick);
        let source_id = self.source_id.clone();

        // Calculate initial state
        let (remaining, progress) = Self::calculate_remaining(period);
        on_tick(remaining, progress);

        // Set up a timer that fires every 100ms for smooth countdown
        let id = glib::timeout_add_local(
            Duration::from_millis(100),
            clone!(
                #[strong]
                on_period_change,
                #[strong]
                on_tick,
                move || {
                    let (remaining, progress) = Self::calculate_remaining(period);

                    // Check if we're at the start of a new period
                    if remaining == period {
                        on_period_change();
                    }

                    on_tick(remaining, progress);
                    glib::ControlFlow::Continue
                }
            ),
        );

        source_id.set(Some(id));
    }

    /// Stop the timer
    pub fn stop(&self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }

    /// Calculate remaining seconds and progress in current TOTP period
    fn calculate_remaining(period: u32) -> (u32, f64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);

        let total_millis = now.as_millis() as u64;
        let period_millis = period as u64 * 1000;
        let elapsed_in_period = (total_millis % period_millis) as u32;
        let remaining_millis = period_millis as u32 - elapsed_in_period;
        let remaining_secs = (remaining_millis + 999) / 1000; // Round up

        // Progress from 1.0 (full) to 0.0 (empty)
        let progress = remaining_millis as f64 / period_millis as f64;

        (remaining_secs, progress)
    }

    /// Get current remaining seconds and progress
    pub fn remaining(&self) -> (u32, f64) {
        Self::calculate_remaining(self.period)
    }

    /// Get the current TOTP counter value
    pub fn current_counter(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        now.as_secs() / self.period as u64
    }
}

impl Default for TimerService {
    fn default() -> Self {
        Self::new(30)
    }
}

impl Drop for TimerService {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_remaining() {
        // This is somewhat difficult to test deterministically
        // Just ensure it returns valid values
        let (remaining, progress) = TimerService::calculate_remaining(30);
        assert!(remaining <= 30);
        assert!(remaining >= 1);
        assert!(progress >= 0.0);
        assert!(progress <= 1.0);
    }
}
