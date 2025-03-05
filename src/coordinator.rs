use std::sync::mpsc::Receiver;
use crate::history::History;
use crate::observations::Observation;

pub fn coordinator(init: u32, receive: Receiver<Observation>) {
    let mut history = History::new();

    loop {
        for observation in receive.try_iter() {
            history.add_new(observation);
        }

        let new_value = history.apply(init);
        println!("Calculated True Value: {:?}", new_value);
    }
}