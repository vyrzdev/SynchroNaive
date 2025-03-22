use rand::prelude::ThreadRng;
use rand::rng;
use crate::observations::Tick;
use crate::observations::DefinitionPredicate;
use crate::observations::DefinitionPredicate::Mutation;
use crate::testing::{exp, Event, Lambda};
use crate::value::Value;

pub struct MockPlatformConfig {
    pub(crate) name: String,
    pub(crate) sale_lambda: Lambda, // Sales Per Tick
    pub(crate) edit_lambda: Lambda, // Manual Edits Per Tick
}

pub struct MockPlatform {
    pub(crate) value: Value,
    pub(crate) config: MockPlatformConfig,
    next_sale: Tick,
    rng: ThreadRng
}

impl MockPlatform {
    pub(crate) fn new(config: MockPlatformConfig, initial_value: Value) -> Self {
        let mut rng = rng();
        let next_sale = exp(config.sale_lambda, &mut rng);;

        MockPlatform {
            value: initial_value,
            config, next_sale, rng
        }
    }

    fn make_sale(&mut self, now: &Tick) -> Event {
        (Mutation {delta: -1}, now.clone())
    }

    pub(crate) fn do_tick(&mut self, now: &Tick) -> Option<(DefinitionPredicate, Tick)> {
        if now >= &self.next_sale {
            let event = self.make_sale(now);
            self.value = event.0.apply(&self.value).unwrap();

            self.next_sale = now + exp(self.config.sale_lambda, &mut self.rng);
            return Some(event);
        }

        return None;
    }
}