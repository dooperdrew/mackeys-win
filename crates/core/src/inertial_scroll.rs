//! Pure physics and delta accumulator model for Apple Magic Mouse inertial scrolling (M18).
//!
//! Conforms to `#![forbid(unsafe_code)]` and zero Win32 dependencies.
//! Provides momentum decay, natural scroll inversion, and smooth wheel delta quantization.

/// Quantized wheel clicks to be injected into the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollDeltas {
    /// Vertical wheel delta (positive = up / away from user, negative = down / toward user).
    pub vertical: i32,
    /// Horizontal wheel delta (positive = right, negative = left).
    pub horizontal: i32,
}

impl ScrollDeltas {
    /// Returns true if neither vertical nor horizontal delta is non-zero.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertical == 0 && self.horizontal == 0
    }
}

/// Standard Windows wheel delta step size (WHEEL_DELTA).
pub const WHEEL_DELTA: i32 = 120;

/// Default decay friction coefficient applied per frame (~60 FPS / 16ms).
pub const DEFAULT_FRICTION: f64 = 0.92;

/// Epsilon threshold below which velocity is clamped to zero.
pub const VELOCITY_EPSILON: f64 = 0.05;

/// Inertial scrolling simulation engine for Apple Magic Mouse and Trackpad touch surfaces.
#[derive(Debug, Clone)]
pub struct InertialScroller {
    /// Current velocity along Y axis.
    velocity_y: f64,
    /// Current velocity along X axis.
    velocity_x: f64,
    /// Sub-pixel accumulator for vertical wheel ticks.
    accumulator_y: f64,
    /// Sub-pixel accumulator for horizontal wheel ticks.
    accumulator_x: f64,
    /// Decay multiplier applied each frame during inertia (0.0 .. 1.0).
    friction: f64,
    /// Speed scale factor (e.g. 1.0 = 100%).
    speed_factor: f64,
    /// Whether macOS natural scrolling direction is enabled.
    natural_scroll: bool,
    /// Whether finger is currently touching the surface.
    finger_down: bool,
}

impl Default for InertialScroller {
    fn default() -> Self {
        Self::new(false, 100)
    }
}

impl InertialScroller {
    /// Creates a new `InertialScroller` with natural scroll and speed percentage.
    #[must_use]
    pub fn new(natural_scroll: bool, speed_percent: u32) -> Self {
        let speed_factor = (speed_percent.clamp(20, 500) as f64) / 100.0;
        Self {
            velocity_y: 0.0,
            velocity_x: 0.0,
            accumulator_y: 0.0,
            accumulator_x: 0.0,
            friction: DEFAULT_FRICTION,
            speed_factor,
            natural_scroll,
            finger_down: false,
        }
    }

    /// Sets whether natural (macOS-style) scroll direction is enabled.
    pub fn set_natural_scroll(&mut self, natural: bool) {
        self.natural_scroll = natural;
    }

    /// Sets the speed multiplier percentage.
    pub fn set_speed_percent(&mut self, speed_percent: u32) {
        self.speed_factor = (speed_percent.clamp(20, 500) as f64) / 100.0;
    }

    /// Resets all velocities, accumulators, and contact states.
    pub fn reset(&mut self) {
        self.velocity_y = 0.0;
        self.velocity_x = 0.0;
        self.accumulator_y = 0.0;
        self.accumulator_x = 0.0;
        self.finger_down = false;
    }

    /// Notifies that a finger made contact with the touch surface, interrupting any ongoing inertia.
    pub fn on_finger_down(&mut self) {
        self.velocity_y = 0.0;
        self.velocity_x = 0.0;
        self.finger_down = true;
    }

    /// Notifies that all fingers lifted from the touch surface, beginning momentum decay.
    pub fn on_finger_up(&mut self) {
        self.finger_down = false;
    }

    /// Processes a raw touch surface delta from hardware (positive dy = downward swipe).
    ///
    /// Computes direct wheel clicks and updates instantaneous velocity.
    pub fn add_surface_delta(&mut self, dy: f64, dx: f64) -> ScrollDeltas {
        self.finger_down = true;

        // In Windows standard scrolling:
        // Swiping downward (toward user) scrolls content downward (negative WHEEL_DELTA).
        // If natural_scroll is true (macOS style): swiping downward pulls content down (positive WHEEL_DELTA).
        let dir_y = if self.natural_scroll { 1.0 } else { -1.0 };
        let dir_x = if self.natural_scroll { -1.0 } else { 1.0 };

        let scaled_dy = dy * self.speed_factor * dir_y;
        let scaled_dx = dx * self.speed_factor * dir_x;

        // Smooth velocity exponential moving average
        self.velocity_y = (self.velocity_y * 0.4) + (scaled_dy * 0.6);
        self.velocity_x = (self.velocity_x * 0.4) + (scaled_dx * 0.6);

        self.accumulator_y += scaled_dy * (WHEEL_DELTA as f64);
        self.accumulator_x += scaled_dx * (WHEEL_DELTA as f64);

        self.drain_accumulators()
    }

    /// Steps the momentum simulation forward by one frame (~16ms).
    ///
    /// Returns `Some(ScrollDeltas)` while momentum is carrying the scroll,
    /// or `None` when velocity has settled to zero or finger is still touching down.
    pub fn step_inertia(&mut self) -> Option<ScrollDeltas> {
        if self.finger_down {
            return None;
        }

        if self.velocity_y.abs() < VELOCITY_EPSILON && self.velocity_x.abs() < VELOCITY_EPSILON {
            self.velocity_y = 0.0;
            self.velocity_x = 0.0;
            return None;
        }

        self.accumulator_y += self.velocity_y * (WHEEL_DELTA as f64);
        self.accumulator_x += self.velocity_x * (WHEEL_DELTA as f64);

        // Apply friction decay
        self.velocity_y *= self.friction;
        self.velocity_x *= self.friction;

        let deltas = self.drain_accumulators();
        if deltas.is_empty() {
            // Keep returning empty deltas until settled or clicks emerge
            Some(ScrollDeltas::default())
        } else {
            Some(deltas)
        }
    }

    /// Drains whole integer increments from accumulators.
    fn drain_accumulators(&mut self) -> ScrollDeltas {
        let mut deltas = ScrollDeltas::default();

        let whole_y = (self.accumulator_y / (WHEEL_DELTA as f64)).trunc() as i32;
        if whole_y != 0 {
            deltas.vertical = whole_y * WHEEL_DELTA;
            self.accumulator_y -= (whole_y * WHEEL_DELTA) as f64;
        }

        let whole_x = (self.accumulator_x / (WHEEL_DELTA as f64)).trunc() as i32;
        if whole_x != 0 {
            deltas.horizontal = whole_x * WHEEL_DELTA;
            self.accumulator_x -= (whole_x * WHEEL_DELTA) as f64;
        }

        deltas
    }

    /// Returns whether momentum decay is currently active and decaying.
    #[must_use]
    pub fn is_inertial_active(&self) -> bool {
        !self.finger_down
            && (self.velocity_y.abs() >= VELOCITY_EPSILON
                || self.velocity_x.abs() >= VELOCITY_EPSILON)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_deltas_empty() {
        let d = ScrollDeltas::default();
        assert!(d.is_empty());
        let d2 = ScrollDeltas {
            vertical: 120,
            horizontal: 0,
        };
        assert!(!d2.is_empty());
    }

    #[test]
    fn test_direct_surface_swiping() {
        let mut scroller = InertialScroller::new(false, 100);

        // Swipe downward by 1.0 unit -> Windows normal: negative vertical delta
        let d = scroller.add_surface_delta(1.0, 0.0);
        assert_eq!(d.vertical, -WHEEL_DELTA);
        assert_eq!(d.horizontal, 0);

        // Swipe upward by 1.0 unit -> Windows normal: positive vertical delta
        let d2 = scroller.add_surface_delta(-1.0, 0.0);
        assert_eq!(d2.vertical, WHEEL_DELTA);
    }

    #[test]
    fn test_natural_scroll_inversion() {
        let mut scroller = InertialScroller::new(true, 100);

        // Natural scroll on downward swipe (1.0) -> positive vertical delta
        let d = scroller.add_surface_delta(1.0, 0.0);
        assert_eq!(d.vertical, WHEEL_DELTA);

        // Natural scroll on upward swipe (-1.0) -> negative vertical delta
        let d2 = scroller.add_surface_delta(-1.0, 0.0);
        assert_eq!(d2.vertical, -WHEEL_DELTA);
    }

    #[test]
    fn test_inertia_decay_and_settling() {
        let mut scroller = InertialScroller::new(false, 100);

        // Rapid swipe
        for _ in 0..5 {
            scroller.add_surface_delta(2.0, 0.0);
        }

        // Lift finger
        scroller.on_finger_up();
        assert!(scroller.is_inertial_active());

        // Step momentum frames until it stops
        let mut total_inertia_clicks = 0;
        let mut frames = 0;
        while let Some(deltas) = scroller.step_inertia() {
            total_inertia_clicks += deltas.vertical;
            frames += 1;
            if frames > 120 {
                panic!("Inertia did not settle within 120 frames!");
            }
        }

        assert!(frames > 3);
        assert!(!scroller.is_inertial_active());
        assert!(total_inertia_clicks <= 0); // Continued downward momentum
    }

    #[test]
    fn test_finger_down_interrupts_inertia() {
        let mut scroller = InertialScroller::new(false, 100);
        scroller.add_surface_delta(3.0, 0.0);
        scroller.on_finger_up();
        assert!(scroller.is_inertial_active());

        // Place finger down again
        scroller.on_finger_down();
        assert!(!scroller.is_inertial_active());
        assert_eq!(scroller.step_inertia(), None);
    }
}
