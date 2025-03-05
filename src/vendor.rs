use std::sync::mpsc;

pub type Config = String;

pub struct Observer {
    uuid: String,
    config: Config,
}


impl Observer {
    pub fn new(config: Config) -> Observer {
        Observer {
            uuid: "".to_string(),
            config: config
        }
    }

    pub fn poll_worker(&self, sync: mpsc::Sender<()>) {

    }
}

