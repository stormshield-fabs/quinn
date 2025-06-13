//! Logic for controlling the rate at which data is sent

use crate::Instant;
use crate::connection::RttEstimator;
use std::sync::Arc;
use std::{any::Any, time::Duration};

mod bbr;
mod cubic;
mod new_reno;

pub use bbr::{Bbr, BbrConfig};
pub use cubic::{Cubic, CubicConfig};
pub use new_reno::{NewReno, NewRenoConfig};
use qlog::events::EventData;

// We don't need to log all qlog metrics every time there is a recovery event.
// Instead, we can log only the MetricsUpdated event data fields that we care
// about, only when they change. To support this, the QLogMetrics structure
// keeps a running picture of the fields.
#[derive(Default, Debug, Clone)]
struct QlogMetrics {
    min_rtt: Option<Duration>,
    smoothed_rtt: Option<Duration>,
    latest_rtt: Option<Duration>,
    rttvar: Option<Duration>,
    cwnd: u64,
    ssthresh: Option<u64>,
    pacing_rate: Option<u64>,
}

impl QlogMetrics {
    // Make a qlog event if the latest instance of QlogMetrics is different.
    //
    // This function diffs each of the fields. A qlog MetricsUpdated event is
    // only generated if at least one field is different. Where fields are
    // different, the qlog event contains the latest value.
    fn maybe_update(&mut self, latest: Self) -> Option<EventData> {
        let mut emit_event = false;

        let new_min_rtt = if self.min_rtt != latest.min_rtt {
            self.min_rtt = latest.min_rtt;
            emit_event = true;
            latest.min_rtt.map(|rtt| rtt.as_secs_f32() * 1000.0)
        } else {
            None
        };

        let new_smoothed_rtt = if self.smoothed_rtt != latest.smoothed_rtt {
            self.smoothed_rtt = latest.smoothed_rtt;
            emit_event = true;
            latest.smoothed_rtt.map(|rtt| rtt.as_secs_f32() * 1000.0)
        } else {
            None
        };

        let new_latest_rtt = if self.latest_rtt != latest.latest_rtt {
            self.latest_rtt = latest.latest_rtt;
            emit_event = true;
            latest.latest_rtt.map(|rtt| rtt.as_secs_f32() * 1000.0)
        } else {
            None
        };

        let new_rttvar = if self.rttvar != latest.rttvar {
            self.rttvar = latest.rttvar;
            emit_event = true;
            latest.rttvar.map(|rtt| rtt.as_secs_f32() * 1000.0)
        } else {
            None
        };

        let new_cwnd = if self.cwnd != latest.cwnd {
            self.cwnd = latest.cwnd;
            emit_event = true;
            Some(latest.cwnd)
        } else {
            None
        };

        let new_ssthresh = if self.ssthresh != latest.ssthresh {
            self.ssthresh = latest.ssthresh;
            emit_event = true;
            latest.ssthresh
        } else {
            None
        };

        let new_pacing_rate = if self.pacing_rate != latest.pacing_rate {
            self.pacing_rate = latest.pacing_rate;
            emit_event = true;
            latest.pacing_rate
        } else {
            None
        };

        if emit_event {
            // QVis can't use all these fields and they can be large.
            return Some(EventData::MetricsUpdated(
                qlog::events::quic::MetricsUpdated {
                    min_rtt: new_min_rtt,
                    smoothed_rtt: new_smoothed_rtt,
                    latest_rtt: new_latest_rtt,
                    rtt_variance: new_rttvar,
                    congestion_window: new_cwnd,
                    ssthresh: new_ssthresh,
                    pacing_rate: new_pacing_rate,
                    ..Default::default()
                },
            ));
        }

        None
    }
}

/// Common interface for different congestion controllers
pub trait Controller: Send + Sync {
    /// One or more packets were just sent
    #[allow(unused_variables)]
    fn on_sent(&mut self, now: Instant, bytes: u64, last_packet_number: u64) {}

    /// Packet deliveries were confirmed
    ///
    /// `app_limited` indicates whether the connection was blocked on outgoing
    /// application data prior to receiving these acknowledgements.
    #[allow(unused_variables)]
    fn on_ack(
        &mut self,
        now: Instant,
        sent: Instant,
        bytes: u64,
        app_limited: bool,
        rtt: &RttEstimator,
    ) {
    }

    /// Packets are acked in batches, all with the same `now` argument. This indicates one of those batches has completed.
    #[allow(unused_variables)]
    fn on_end_acks(
        &mut self,
        now: Instant,
        in_flight: u64,
        app_limited: bool,
        largest_packet_num_acked: Option<u64>,
    ) {
    }

    /// Packets were deemed lost or marked congested
    ///
    /// `in_persistent_congestion` indicates whether all packets sent within the persistent
    /// congestion threshold period ending when the most recent packet in this batch was sent were
    /// lost.
    /// `lost_bytes` indicates how many bytes were lost. This value will be 0 for ECN triggers.
    fn on_congestion_event(
        &mut self,
        now: Instant,
        sent: Instant,
        is_persistent_congestion: bool,
        lost_bytes: u64,
    );

    /// The known MTU for the current network path has been updated
    fn on_mtu_update(&mut self, new_mtu: u16);

    /// Number of ack-eliciting bytes that may be in flight
    fn window(&self) -> u64;

    /// Duplicate the controller's state
    fn clone_box(&self) -> Box<dyn Controller>;

    /// Initial congestion window
    fn initial_window(&self) -> u64;

    /// Returns Self for use in down-casting to extract implementation details
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Emits a qlog `MetricsUpdated` event
    fn qlog(&mut self) -> Option<EventData> {
        None
    }
}

/// Constructs controllers on demand
pub trait ControllerFactory {
    /// Construct a fresh `Controller`
    fn build(self: Arc<Self>, now: Instant, current_mtu: u16) -> Box<dyn Controller>;
}

const BASE_DATAGRAM_SIZE: u64 = 1200;
