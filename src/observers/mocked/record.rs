use std::cmp::{max, min};
use log::info;
use rand::rng;
use crate::inference::interval::{Interval, Moment};
use crate::observations::{DefinitionPredicate, Observation, PollingInterpretation, SourceKind, Tick};
use crate::observations::DefinitionPredicate::{Assignment, Mutation, Transition};
use crate::observers::mocked::polling::{ActivePollState, HistoricPollState};
use crate::observers::mocked::record_platform::MockRecordPlatform;
use crate::testing::{norm, Event, Lambda};

pub struct MockRecordPoller {
    rtt_lambda: Lambda,
    rtt_std_dev: Lambda,
    backoff: Tick,
    clock_precision: Tick,
    max_deviation: i64,
    min_deviation: i64,
    deviation_state: DeviationState,
    poll_state: RecordPollState
}
#[derive(Debug)]
pub struct MockObservation {
    interval: (Tick, Tick),
    definition: DefinitionPredicate,
}

pub struct DeviationState {
    send_at: Tick,
    process_at: Tick,
    returned: Option<Tick>,
    reply_at: Tick,
}

pub struct RecordPollState {
    send_at: Tick,
    process_at: Tick,
    returned: Option<Vec<Event>>,
    reply_at: Tick
}


impl MockRecordPoller {
    pub(crate) fn new(rtt_lambda: Lambda, rtt_std_dev: Lambda, backoff: Tick, clock_precision: Tick) -> Self {

        let poll_process_at = norm(rtt_lambda/2.0, rtt_std_dev, &mut rng());
        let poll_reply_at = poll_process_at + norm(rtt_lambda/2.0, rtt_std_dev, &mut rng());

        let deviation_process_at = norm(rtt_lambda/2.0, rtt_std_dev, &mut rng());
        let deviation_reply_at = deviation_process_at + norm(rtt_lambda/2.0, rtt_std_dev, &mut rng());

        MockRecordPoller {
            rtt_lambda,
            rtt_std_dev,
            backoff,
            clock_precision,
            max_deviation: i64::MIN,
            min_deviation: i64::MAX,
            deviation_state: DeviationState {
                send_at: 0,
                process_at: deviation_process_at,
                reply_at: deviation_reply_at,
                returned: None
            },
            poll_state: RecordPollState {
                send_at: 0,
                process_at: poll_process_at,
                returned: None,
                reply_at: poll_reply_at,
            },
        }
    }


    pub(crate) fn do_tick(&mut self, now: &Tick, platform: &mut MockRecordPlatform) -> Option<Vec<Observation>> {
        let mut ret = None;

        if now == &self.deviation_state.process_at {
            self.deviation_state.returned = Some(platform.get_deviating_clock(now));
        }

        if now == &self.deviation_state.reply_at {
            // TODO: Deviation Calculations
            let observed_timestamp = self.deviation_state.returned.unwrap();
            let p_max = (observed_timestamp + self.clock_precision) as i64;
            let p_min = observed_timestamp as i64;
            // info!("p_max: {}", p_max);
            // info!("p_min: {}", p_min);

            let observed_max_deviation = p_max - (self.deviation_state.send_at as i64);
            let observed_min_deviation = p_min - (self.deviation_state.reply_at as i64);

            // info!("Deviation Observed: {observed_timestamp} - Known Between: {}, {}", self.deviation_state.send_at, self.deviation_state.reply_at);
            // info!("Determined Max Possible: {observed_min_deviation} {observed_max_deviation}");
            self.max_deviation = max(self.max_deviation, observed_max_deviation);
            self.min_deviation = min(self.min_deviation, observed_min_deviation);

            let new_send_at = now + self.backoff;
            let new_process_at = new_send_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());
            let new_reply_at = new_process_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());

            self.deviation_state = DeviationState {
                send_at: new_send_at,
                process_at: new_process_at,
                returned: None,
                reply_at: new_reply_at,
            }
        }

        if &self.poll_state.process_at == now {
            self.poll_state.returned = Some(platform.events.clone());
            platform.events = Vec::new();
        }

        if &self.poll_state.reply_at == now {
            if self.poll_state.returned.as_ref().is_some_and(|x| !x.is_empty()) {
                let mut build = Vec::new();
                for (definition, timestamp) in self.poll_state.returned.as_ref().unwrap() {
                    // info!("Observed Timestamp: {timestamp} - Known Deviation: {}, {}", self.min_deviation, self.max_deviation);
                    let max_timestamp = ((timestamp.clone() as i64) - self.min_deviation) as Tick;
                    let min_timestamp = ((timestamp.clone() as i64) - self.max_deviation) as Tick;
                    // info!("Calculated Uncertainty: {min_timestamp} - {max_timestamp}");

                    build.push(Observation {
                        definition: definition.clone(),
                        interval: Interval(Moment(min_timestamp), Moment(max_timestamp)),
                        source: SourceKind::Record(platform.config.name.clone())
                    });
                }
                ret = Some(build);
            }

            // Schedule next poll
            let new_send_at = now + self.backoff;
            let new_process_at = new_send_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());
            let new_reply_at = new_process_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());
            self.poll_state = RecordPollState {
                send_at: new_send_at,
                process_at: new_process_at,
                returned: None,
                reply_at: new_reply_at,
            }
        }
        return ret;
    }
}