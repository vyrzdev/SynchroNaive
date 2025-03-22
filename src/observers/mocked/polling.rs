use rand::rng;
use crate::inference::interval::{Interval, Moment};
use crate::observations::{Observation, SourceKind, Tick};
use crate::observations::{DefinitionPredicate, PollingInterpretation};
use crate::observations::DefinitionPredicate::{Assignment, Mutation, Transition};
use crate::observers::mocked::poll_platform::MockPlatform;
use crate::testing::{norm, Lambda};
use crate::value::Value;

pub struct ActivePollState {
    pub(crate) send_at: Tick,
    pub(crate) process_at: Tick,
    value: Option<Value>,
    pub(crate) reply_at: Tick
}

pub struct HistoricPollState {
    pub(crate) sent: Tick,
    pub(crate) process: Tick,
    pub(crate) value: Value,
    pub(crate) replied: Tick
}

pub struct MockPoller {
    pub(crate) current: ActivePollState,
    pub(crate) last: Option<HistoricPollState>,
    pub(crate) rtt_lambda: Lambda,
    pub(crate) rtt_std_dev: Lambda,
    backoff: Tick,
    interpretation: PollingInterpretation,
}
#[derive(Debug)]
pub struct MockObservation {
    interval: (Tick, Tick),
    definition: DefinitionPredicate,
}

impl MockPoller {
    pub(crate) fn new(rtt_lambda: Lambda, rtt_std_dev: Lambda, backoff: Tick, interpretation: PollingInterpretation) -> Self {
        let next_send_at = 0;
        let next_process_at = next_send_at + norm((rtt_lambda/2.0) as f64, rtt_std_dev, &mut rng());
        let next_reply_at = next_process_at + norm((rtt_lambda/2.0) as f64, rtt_std_dev, &mut rng());
        MockPoller {
            current: ActivePollState {
                send_at: next_send_at,
                process_at: next_process_at,
                value: None,
                reply_at: next_reply_at,
            },
            last: None,
            rtt_lambda, rtt_std_dev, backoff, interpretation
        }
    }


    pub(crate) fn do_tick(&mut self, now: &Tick, platform: &MockPlatform) -> Option<Observation> {
        let mut ret = None;

        if &self.current.send_at == now {}
        if &self.current.process_at == now {
            self.current.value = Some(platform.value.clone());
        }
        if &self.current.reply_at == now {
            if self.last.as_ref().is_some_and(
                |x| &x.value != &self.current.value.unwrap()
            ) {
                ret = Some(Observation {
                    interval: Interval(Moment(self.last.as_ref().unwrap().sent.clone()), Moment(self.current.reply_at.clone())),
                    definition: match self.interpretation {
                        PollingInterpretation::Mutation => {
                            Mutation {delta: self.current.value.unwrap().clone() - self.last.as_ref().unwrap().value.clone() }
                        }
                        PollingInterpretation::Assignment => {
                            Assignment {v_new: self.current.value.unwrap().clone()}
                        }
                        PollingInterpretation::Transition => {
                            Transition {
                                v_0: self.last.as_ref().unwrap().value.clone(),
                                v_1: self.current.value.unwrap().clone()
                            }
                        }
                    },
                    source: SourceKind::Polling(platform.config.name.clone())
                });
            }

            self.last = Some(HistoricPollState {
                sent: self.current.send_at,
                process: self.current.process_at,
                replied: self.current.reply_at,
                value: self.current.value.unwrap()
            });
            let next_send_at = now + self.backoff;
            let next_process_at = next_send_at + norm((self.rtt_lambda/2.0) as f64, self.rtt_std_dev, &mut rng());
            let next_reply_at = next_process_at + norm((self.rtt_lambda/2.0) as f64, self.rtt_std_dev, &mut rng());
            self.current = ActivePollState {
                send_at: next_send_at,
                process_at: next_process_at,
                value: None,
                reply_at: next_reply_at,
            };
        }
        return ret;
    }
}