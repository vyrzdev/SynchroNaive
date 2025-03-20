use log::{error, info};
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch;
use crate::history::History;
use crate::observations::Observation;
use crate::value::Value;

const PROCESS_BUFFER_LIMIT: usize = 100;

pub async fn coordinator(init_consensus: Option<Value>, mut receive: Receiver<Observation>, w_tx: watch::Sender<Option<Value>>) -> ! {
    let mut history = History::new(); // Initialise empty O(V).
    let mut observations = Vec::with_capacity(PROCESS_BUFFER_LIMIT); // Input buffer to read observations.

    loop {
        receive.recv_many(&mut observations, PROCESS_BUFFER_LIMIT).await; // Recieve all pending observations into buffer.

        for observation in observations.drain(0..observations.len()) {
            history.add_new(observation); // For each observation, add it to O(V).
        }

        let new_value = history.apply(init_consensus);

        if let Ok(v) = new_value {
            info!("Coordinator - New Value: {:?}", new_value);
            w_tx.send(v).unwrap();
        } else {
            error!("Coordinator - Error: {:?}", new_value);
        }
    }
}