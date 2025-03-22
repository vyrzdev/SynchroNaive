use rand::prelude::ThreadRng;
use rand::rng;
use crate::observations::Tick;
use crate::observations::DefinitionPredicate;
use crate::observations::DefinitionPredicate::Mutation;
use crate::testing::{exp, norm, Event, Lambda};
use crate::value::Value;

pub struct MockRecordPlatformConfig {
    pub(crate) name: String,
    pub(crate) sale_lambda: Lambda, // Sales Per Tick
    pub(crate) edit_lambda: Lambda, // Manual Edits Per Tick
    pub(crate) deviation_lambda: Lambda, // Average Tick Deviation between observer and platform.
    pub(crate) deviation_std_dev: Lambda, // Std Deviation from that (stablility)
    pub(crate) clock_precision: Tick // In significant Figures?
}

pub struct MockRecordPlatform {
    pub(crate) value: Value,
    pub(crate) events: Vec<(DefinitionPredicate, Tick)>,
    pub(crate) config: MockRecordPlatformConfig,
    next_sale: Tick,
    rng: ThreadRng
}

impl MockRecordPlatform {
    pub(crate) fn new(config: MockRecordPlatformConfig, initial_value: Value) -> Self {
        let mut rng = rng();
        let next_sale = exp(config.sale_lambda, &mut rng);;

        MockRecordPlatform {
            value: initial_value,
            events: Vec::new(),
            config, next_sale, rng
        }
    }

    pub fn get_deviating_clock(&mut self, now: &Tick) -> Tick {
        return (((now + norm(self.config.deviation_lambda, self.config.deviation_std_dev, &mut self.rng)) as Tick) / self.config.clock_precision) * self.config.clock_precision;
    }

    fn make_sale(&mut self, now: &Tick) -> Event {
        (Mutation {delta: -1}, now.clone())
    }

    pub(crate) fn do_tick(&mut self, now: &Tick) -> Option<(DefinitionPredicate, Tick)> {
        if now >= &self.next_sale {
            let event = self.make_sale(now);
            self.value = event.0.apply(&self.value).unwrap();

            let deviating_clock = self.get_deviating_clock(now);
            self.events.push((event.0.clone(), deviating_clock));

            self.next_sale = now + exp(self.config.sale_lambda, &mut self.rng);
            return Some(event);
        }

        return None;
    }
}