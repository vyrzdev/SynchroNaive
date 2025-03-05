use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver;
use crate::history::History;
use crate::observations::Observation;
use crate::value::Value;

const PROCESS_BUFFER_LIMIT: usize = 100;

pub async fn coordinator(init: Value, mut receive: Receiver<Observation>) -> ! {
    let mut history = History::new();
    let mut observations = Vec::with_capacity(PROCESS_BUFFER_LIMIT);

    loop {
        receive.recv_many(&mut observations, PROCESS_BUFFER_LIMIT).await;

        for observation in observations.drain(0..observations.len()) {
            history.add_new(observation);
        }

        let new_value = history.apply(init.clone());
        println!("Calculated True Value: {:?}", new_value);
    }
}