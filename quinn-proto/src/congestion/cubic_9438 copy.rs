use std::time::Duration;

use crate::congestion::Controller;

struct Cubic {
    /// Smoothed RTT in seconds, calculated as described in RFC6298
    rtt: f64,

    /// Current congestion window in segments
    cwnd: f64,

    /// Current slow start threshold in segments
    ssthresh: f64,

    /// Congestion window in segments at the time of setting sshtresh most recently
    ///
    /// This is either upon exiting the first slow start, or just before cwnd was reduced in the last congestion event.
    cwnd_prior: f64,

    /// Congestion window in segments hust before cwnd was reduced in the last congestion event, when fast convergence is disabled (same as `cwnd_prior` on a congestion event).
    ///
    /// Maybe be further reduced if fast convergence is enabled, based on the current saturation point.
    w_max: f64,

    /// The time period in seconds it takes to increase the congestion window size at the beginning of the current congestion avoidance stage to `w_max`.
    k: f64,

    /// The time in seconds at which the current congestion avoidance stage started
    t_epoch: f64,

    /// Congestion window at the beginning of the current congestion avoidance stage, i.e. at `t_epoch`.
    cwnd_epoch: f64,

    /// Congestion window in segments at time `t` in seconds
    w_cubic: u64,

    /// Target value of the congestion window in segments after the next RTT
    target: u64,

    /// Estiamte for the congestion window in segments in the Reno-friendly region
    w_est: f64,

    /// Number of SMSS-sized segments acked when a new ACK is received
    segments_acked: f64,
}

impl Cubic {
    /// CUBIC multiplicative decrease factor
    ///
    /// 4.6.  Multiplicative Decrease
    const BETA_CUBIC: f64 = 0.7;

    /// CUBIC additive increase factor used in the Reno-friendly region
    ///
    /// §4.3. Reno-Friendly Region
    // FIXME: should be rest to 1 when w_est >= cwnd_prior
    const ALPHA_CUBIC: f64 = 3.0 * ((1.0 - Self::BETA_CUBIC) / (1.0 + Self::BETA_CUBIC));

    /// Aggressiveness of CUBIC in competing with other congestion control algorithms in high-BDP networks
    ///
    /// 5.1. Fairness to Reno
    const C: f64 = 0.4;

    /// Window increase function
    ///
    /// - `t` is the elapsed time in seconds from the beginning of the current congestion avoidance stage
    fn w_cubic(&self, t: Duration) -> f64 {
        Self::C * (t.as_secs_f64() - self.k()).powi(3) + self.w_max
    }

    fn k(&self) -> f64 {
        ((self.w_max - self.cwnd_epoch) / Self::C).cbrt()
    }

    /// Estimate the window size in the Reno-friendly region
    ///
    /// §4.3. Reno-Friendly Region, Figure 4.
    fn w_est(&self) -> f64 {
        self.w_est + Self::ALPHA_CUBIC * (self.segments_acked / self.cwnd)
    }

    /// 4.2. Window Increase Function
    // TODO: t MUST NOT include periods during which cwnd has not been updated due to application-limited behavior.
    fn target(&self, t: Duration, rtt: Duration) -> f64 {
        let w_cubic = self.w_cubic(t.saturating_add(rtt));

        if w_cubic < self.cwnd {
            self.cwnd
        } else if w_cubic > 1.5 * self.cwnd {
            1.5 * self.cwnd
        } else {
            w_cubic
        }
    }
}

impl Controller for Cubic {
    fn on_ack(
        &mut self,
        now: std::time::Instant,
        sent: std::time::Instant,
        bytes: u64,
        app_limited: bool,
        rtt: &crate::RttEstimator,
    ) {
        if self.cwnd < self.ssthresh {
            // RFC 9002 §7.3.1. Slow Start
            // While a sender is in slow start, the congestion window increases by 
            // the number of bytes acknowledged when each acknowledgment is 
            // processed. This results in exponential growth of the congestion window.
            // TODO: implement HyStart++
            // 4.10. Slow Start
            self.cwnd += bytes as f64;
            return;
        }

        // 4.4.  Concave Region
        // 4.5.  Convex Region
        self.cwnd += (self.target(todo!(), rtt.get()) - self.cwnd) / self.cwnd;

        // TODO: spurious loss detection
        // restore undo values unless cwnd is higher than cwnd_priori
    }

    fn on_congestion_event(
        &mut self,
        now: std::time::Instant,
        sent: std::time::Instant,
        is_persistent_congestion: bool,
        lost_bytes: u64,
    ) {
        // TODO: 4.9.2. Spurious Fast Retransmits

        // 4.7. Fast Convergence
        self.w_max = if self.cwnd < self.w_max {
            // TODO: add a setting to disable fast convergence
            self.cwnd * ((1.0 + Self::BETA_CUBIC) / 2.0)
        } else {
            self.cwnd
        };

        let flight_size: f64 = todo!();

        // 4.6.  Multiplicative Decrease
        self.ssthresh = flight_size * Self::BETA_CUBIC;

        self.cwnd_prior = self.cwnd;

        let min_cwnd = 2.0; // on loss
        let min_cwnd = 1.0; // on ECE

        self.cwnd = self.ssthresh.max(min_cwnd);

        self.ssthresh = self.ssthresh.max(2.0);

        todo!()

        // TODO: implement RFC7661 to support application-limited traffic
        // TODO: reduce cwnd further than 1 SMSS if the congestion event persists, cf. RFC3168
    }

    fn on_mtu_update(&mut self, new_mtu: u16) {
        todo!()
    }

    fn window(&self) -> u64 {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Controller> {
        todo!()
    }

    fn initial_window(&self) -> u64 {
        todo!()
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        todo!()
    }
}
