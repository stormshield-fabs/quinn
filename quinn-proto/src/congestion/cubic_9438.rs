use crate::congestion::Controller;

struct Cubic {}

impl Controller for Cubic {
    fn on_sent(&mut self, now: std::time::Instant, bytes: u64, last_packet_number: u64) {}

    fn on_ack(
        &mut self,
        now: std::time::Instant,
        sent: std::time::Instant,
        bytes: u64,
        app_limited: bool,
        rtt: &crate::RttEstimator,
    ) {
    }

    fn on_end_acks(
        &mut self,
        now: std::time::Instant,
        in_flight: u64,
        app_limited: bool,
        largest_packet_num_acked: Option<u64>,
    ) {
    }

    fn on_congestion_event(
        &mut self,
        now: std::time::Instant,
        sent: std::time::Instant,
        is_persistent_congestion: bool,
        lost_bytes: u64,
    ) {
        todo!()
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
