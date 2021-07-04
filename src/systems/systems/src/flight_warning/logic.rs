use crate::simulation::UpdateContext;
use std::time::Duration;

/// A confirmation circuit, which only passes a signal once it has been stable for a certain amount
/// of time. It is inspired by the CONF nodes as used by the FWC. When it detects either a rising or
/// falling edge (depending on it's type) it will wait for up to time t and emit the  incoming
/// signal if it was stable throughout t. If at any point the signal reverts during t the state is
/// fully reset, and the original signal will be emitted again.
pub struct ConfirmationNode {
    leading_edge: bool,
    time_delay: Duration,
    condition_since: Duration,
    output: bool,
}

impl ConfirmationNode {
    pub fn new(leading_edge: bool, time_delay: Duration) -> Self {
        Self {
            leading_edge,
            time_delay,
            condition_since: Duration::from_secs(0),
            output: false,
        }
    }

    pub fn new_leading(time_delay: Duration) -> Self {
        Self::new(true, time_delay)
    }

    pub fn new_falling(time_delay: Duration) -> Self {
        Self::new(false, time_delay)
    }

    pub fn update(&mut self, context: &UpdateContext, hi: bool) -> bool {
        let condition_met = hi == self.leading_edge;
        if condition_met {
            self.condition_since += context.delta();
            self.output = self.condition_since >= self.time_delay;
        } else {
            self.condition_since = Duration::from_secs(0);
            self.output = false;
        }
        return self.output;
    }
}

/// A monostable trigger, which outputs lo until it detects a rising or falling edge (depending on
/// it's type). At that point it will start outputting hi until the time_delay has elapsed. If the
/// node is retriggerable, a matching edge will reset the timer. Otherwise they are ignored until
/// time_delay has elapsed and the node outputs lo again.
pub struct MonostableTriggerNode {
    leading_edge: bool,
    time_delay: Duration,
    retriggerable: bool,
    remaining_trigger: Duration,
    last_hi: bool,
    output: bool,
}

impl MonostableTriggerNode {
    pub fn new(leading_edge: bool, time_delay: Duration) -> Self {
        Self {
            leading_edge,
            time_delay,
            retriggerable: false,
            remaining_trigger: Duration::from_secs(0),
            last_hi: false,
            output: false,
        }
    }

    pub fn new_retriggerable(leading_edge: bool, time_delay: Duration) -> Self {
        Self {
            leading_edge,
            time_delay,
            retriggerable: true,
            remaining_trigger: Duration::from_secs(0),
            last_hi: false,
            output: false,
        }
    }

    pub fn update(&mut self, context: &UpdateContext, hi: bool) -> bool {
        self.remaining_trigger = match self.remaining_trigger.checked_sub(context.delta()) {
            Some(res) => res,
            None => Duration::from_secs(0),
        };
        if self.retriggerable || self.remaining_trigger == Duration::from_secs(0) {
            let condition_met = self.last_hi != hi && hi == self.leading_edge;
            if condition_met {
                self.remaining_trigger = self.time_delay.clone();
            }
        }
        self.last_hi = hi;
        self.output = self.remaining_trigger > Duration::from_secs(0);
        return self.output;
    }
}

/// A flip-flop or memory circuit that can be used to store a single bit. It has two inputs: Set and
/// Reset. At first it will always emit a falsy value, until it receives a signal on the set input,
/// at which point it will start emitting a truthy value. This will continue until a signal is
/// received on the reset input, at which point it reverts to the original falsy output. If a signal
/// is received on both set and reset at the same time, the input with a star will have precedence.
/// The NVM flag is not implemented right now but can be used to indicate non-volatile memory
/// storage,  which means the value will persist even when electrical power is lost and subsequently
/// restored.
pub struct MemoryNode {
    has_set_precedence: bool,
    nvm: bool,
    pub output: bool,
}

impl MemoryNode {
    pub fn new(has_set_precedence: bool) -> Self {
        Self {
            has_set_precedence,
            nvm: false,
            output: false,
        }
    }

    pub fn new_nvm(has_set_precedence: bool) -> Self {
        Self {
            has_set_precedence,
            nvm: true,
            output: false,
        }
    }

    pub fn update(&mut self, _context: &UpdateContext, set: bool, reset: bool) -> bool {
        self.output = if set && reset {
            self.has_set_precedence
        } else if set {
            true
        } else if reset {
            false
        } else {
            self.output
        };
        return self.output;
    }
}

/// A hysteresis circuit, which will switch between a high and a low state based on two different
/// numerical comparisons to prevent rapid output switching due to minor value fluctuations.
/// The circuit will output lo until the up condition is met (value >= up). Then, even if the
/// condition subsequently fails, the circuit will continue outputting hi until the down condition
/// is met (value <= dn).
pub struct HysteresisNode<T> {
    up: T,
    dn: T,
    output: bool,
}

impl<T> HysteresisNode<T>
where
    T: PartialOrd,
{
    pub fn new(dn: T, up: T) -> Self {
        Self {
            up,
            dn,
            output: false,
        }
    }

    pub fn update(&mut self, _context: &UpdateContext, value: T) -> bool {
        if self.output {
            self.output = if value <= self.dn { false } else { self.output }
        } else {
            self.output = if value >= self.up { true } else { self.output }
        }
        return self.output;
    }
}

/// A node that memorizes the value from the preceding call.
pub struct PreceedingValueNode {
    predecessor: bool,
}

impl PreceedingValueNode {
    pub fn new() -> Self {
        Self { predecessor: false }
    }

    pub fn get(&self) -> bool {
        self.predecessor
    }

    pub fn update(&mut self, _context: &UpdateContext, value: bool) {
        self.predecessor = value;
    }
}

/// A circuit that emits a specific signal when the input changes from hi to lo or from lo to hi.
/// The output can be chosen to be hi or lo, so that either:
/// 1) the stable output signal is lo, and during a change the output change signal is hi
/// 2) the stable output signal is hi, and during a change the output change signal is lo
pub struct TransientDetectionNode {
    change_signal: bool,
    predecessor: bool,
}

impl TransientDetectionNode {
    pub fn new(change_signal: bool) -> Self {
        Self {
            change_signal: change_signal,
            predecessor: false,
        }
    }

    pub fn update(&mut self, _context: &UpdateContext, value: bool) -> bool {
        let predecessor = self.predecessor;
        self.predecessor = value;
        return if value != predecessor {
            // A change has occurred
            self.change_signal
        } else {
            // No change has occurred
            !self.change_signal
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::f64::*;
    use uom::si::{
        acceleration::foot_per_second_squared, length::foot,
        thermodynamic_temperature::degree_celsius, velocity::knot,
    };

    #[cfg(test)]
    mod confirmation_node_tests {
        use super::*;

        #[test]
        fn when_condition_fails_stays_lo() {
            let mut node = ConfirmationNode::new_leading(Duration::from_secs(1));
            assert_eq!(node.update(&context(Duration::from_secs(1)), false), false);
        }

        #[test]
        fn when_condition_passes_first_then_output_lo() {
            let mut node = ConfirmationNode::new_leading(Duration::from_secs(1));
            assert_eq!(
                node.update(&context(Duration::from_secs_f64(0.1)), true),
                false
            );
        }

        #[test]
        fn when_condition_passes_long_enough_then_output_hi() {
            let mut node = ConfirmationNode::new_leading(Duration::from_secs(1));
            assert_eq!(node.update(&context(Duration::from_secs(1)), true), true);
        }

        #[test]
        fn when_condition_resets_then_output_hi() {
            let mut node = ConfirmationNode::new_leading(Duration::from_secs(1));
            node.update(&context(Duration::from_secs(1)), true);
            node.update(&context(Duration::from_secs_f64(0.1)), false);
            assert_eq!(
                node.update(&context(Duration::from_secs_f64(0.1)), true),
                false
            );
        }
    }

    #[cfg(test)]
    mod monostable_trigges_node_tests {
        use super::*;

        #[test]
        fn when_created_outputs_lo() {
            let mut node = MonostableTriggerNode::new(true, Duration::from_secs(1));
            assert_eq!(node.update(&context(Duration::from_secs(1)), false), false);
        }

        #[test]
        fn when_triggered_outputs_hi() {
            let mut node = MonostableTriggerNode::new(true, Duration::from_secs(1));
            assert_eq!(node.update(&context(Duration::from_secs(1)), true), true);
        }

        #[test]
        fn when_triggered_and_elapses_outputs_lo() {
            let mut node = MonostableTriggerNode::new(true, Duration::from_secs(1));
            node.update(&context(Duration::from_secs(1)), true);
            assert_eq!(node.update(&context(Duration::from_secs(1)), false), false);
        }

        #[test]
        fn when_retriggered_and_elapses_outputs_lo() {
            let mut node = MonostableTriggerNode::new(true, Duration::from_secs(1));
            node.update(&context(Duration::from_secs(1)), true);
            node.update(&context(Duration::from_secs_f64(0.5)), false);
            node.update(&context(Duration::from_secs_f64(0.4)), true);
            assert_eq!(
                node.update(&context(Duration::from_secs_f64(0.1)), false),
                false
            );
        }

        #[test]
        fn when_retriggerable_retriggered_and_elapses_outputs_lo() {
            let mut node = MonostableTriggerNode::new_retriggerable(true, Duration::from_secs(1));
            node.update(&context(Duration::from_secs(1)), true);
            node.update(&context(Duration::from_secs_f64(0.5)), false);
            node.update(&context(Duration::from_secs_f64(0.4)), true);
            assert_eq!(
                node.update(&context(Duration::from_secs_f64(0.1)), false),
                true
            );
        }
    }

    #[cfg(test)]
    mod memory_node_tests {
        use super::*;

        #[test]
        fn when_created_outputs_lo() {
            let mut node = MemoryNode::new(true);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), false, false),
                false
            );
        }

        #[test]
        fn when_set_outputs_lo() {
            let mut node = MemoryNode::new(true);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), true, false),
                true
            );
        }

        #[test]
        fn when_set_keeps_lo() {
            let mut node = MemoryNode::new(true);
            node.update(&context(Duration::from_secs(1)), true, false);

            assert_eq!(
                node.update(&context(Duration::from_secs(1)), false, false),
                true
            );
        }

        #[test]
        fn when_reset_outputs_lo() {
            let mut node = MemoryNode::new(true);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), false, true),
                false
            );

            node.update(&context(Duration::from_secs(1)), true, false);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), false, true),
                false
            );
        }

        #[test]
        fn when_set_precedence_and_both_hi_outputs_hi() {
            let mut node = MemoryNode::new(true);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), true, true),
                true
            );
        }

        #[test]
        fn when_no_set_precedence_and_both_hi_outputs_lo() {
            let mut node = MemoryNode::new(false);
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), true, true),
                false
            );
        }
    }

    #[cfg(test)]
    mod hystereses_node_tests {
        use super::*;

        #[test]
        fn when_stays_below_up_stays_lo() {
            let mut node =
                HysteresisNode::new(Length::new::<foot>(-10.0), Length::new::<foot>(10.0));
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), Length::new::<foot>(9.9)),
                false
            );
        }

        #[test]
        fn when_above_up_becomes_hi() {
            let mut node =
                HysteresisNode::new(Length::new::<foot>(-10.0), Length::new::<foot>(10.0));
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), Length::new::<foot>(10.0)),
                true
            );
        }

        #[test]
        fn when_exceeds_up_stays_hi() {
            let mut node =
                HysteresisNode::new(Length::new::<foot>(-10.0), Length::new::<foot>(10.0));
            node.update(&context(Duration::from_secs(1)), Length::new::<foot>(10.0));
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), Length::new::<foot>(9.9)),
                true
            );
        }

        #[test]
        fn when_falls_below_dn_returns_to_lo() {
            let mut node =
                HysteresisNode::new(Length::new::<foot>(-10.0), Length::new::<foot>(10.0));
            node.update(&context(Duration::from_secs(1)), Length::new::<foot>(10.0));
            assert_eq!(
                node.update(&context(Duration::from_secs(1)), Length::new::<foot>(-10.0)),
                false
            );
        }
    }

    #[cfg(test)]
    mod preceeding_value_node_tests {
        use super::*;

        #[test]
        fn when_lo_then_lo() {
            let mut node = PreceedingValueNode::new();
            node.update(&context(Duration::from_secs(1)), false);
            assert_eq!(node.get(), false);
        }

        #[test]
        fn when_hi_then_hi() {
            let mut node = PreceedingValueNode::new();
            node.update(&context(Duration::from_secs(1)), true);
            assert_eq!(node.get(), true);
        }
    }

    #[cfg(test)]
    mod transient_detection_node_tests {
        use super::*;

        #[test]
        fn when_lo_then_lo_emits_lo() {
            let mut node = TransientDetectionNode::new(true);
            node.update(&context(Duration::from_secs(1)), false);
            assert_eq!(node.update(&context(Duration::from_secs(1)), false), false);
        }

        #[test]
        fn when_lo_then_hi_emits_hi() {
            let mut node = TransientDetectionNode::new(true);
            node.update(&context(Duration::from_secs(1)), false);
            assert_eq!(node.update(&context(Duration::from_secs(1)), true), true);
        }

        #[test]
        fn when_hi_then_hi_emits_lo() {
            let mut node = TransientDetectionNode::new(true);
            node.update(&context(Duration::from_secs(1)), true);
            assert_eq!(node.update(&context(Duration::from_secs(1)), true), false);
        }

        #[test]
        fn when_hi_then_lo_emits_hi() {
            let mut node = TransientDetectionNode::new(true);
            node.update(&context(Duration::from_secs(1)), true);
            assert_eq!(node.update(&context(Duration::from_secs(1)), false), true);
        }
    }

    fn context(delta_time: Duration) -> UpdateContext {
        UpdateContext::new(
            delta_time,
            Velocity::new::<knot>(250.),
            Length::new::<foot>(5000.),
            ThermodynamicTemperature::new::<degree_celsius>(25.0),
            true,
            Acceleration::new::<foot_per_second_squared>(0.),
        )
    }
}