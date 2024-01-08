use std::time::Instant;

pub mod input_method_v1;
pub mod input_method_v2;

#[derive(Clone, Copy)]
pub struct RepeatInfo {
    /// The rate of repeating keys in characters per second
    rate: i32,
    /// Delay in milliseconds since key down until repeating starts
    delay: i32,
}

#[derive(Clone, Copy)]
pub enum PressState {
    /// User is pressing no key, or user lifted last pressed key. But kime-wayland is ready for key
    /// long-press.
    NotPressing,
    /// User is pressing a key.
    Pressing {
        /// User started pressing a key at this moment.
        pressed_at: Instant,
        /// `false` if user just started pressing a key. Soon, key repeating will be begin. `true`
        /// if user have pressed a key for a long enough time, key repeating is happening right
        /// now.
        is_repeating: bool,

        /// Key code used by wayland
        key: u32,
        /// Timestamp with millisecond granularity used by wayland. Their base is undefined, so
        /// they can't be compared against system time (as obtained with clock_gettime or
        /// gettimeofday). They can be compared with each other though, and for instance be used to
        /// identify sequences of button presses as double or triple clicks.
        ///
        /// #### Reference
        /// - https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-Input
        wayland_time: u32,
    },
}

impl PressState {
    pub fn is_pressing(&self, query_key: u32) -> bool {
        if let PressState::Pressing { key, .. } = self {
            *key == query_key
        } else {
            false
        }
    }
}
