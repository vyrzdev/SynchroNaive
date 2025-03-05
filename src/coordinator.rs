use std::sync::mpsc::Receiver;
use crate::history::History;
use crate::observations::Observation;

pub fn coordinator(init: u32, receive: Receiver<Observation>) {
    let mut history = History::new();

    loop {
        for observation in receive.try_iter() {
            if let Some(obs) = observation {
                history.add_new(obs);
            } else {
                break;
            }
        }

        let new_value = history.apply(init);
    }
}