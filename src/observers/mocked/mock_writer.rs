use futures::poll;
use rand::rng;
use crate::observations::Tick;
use crate::observers::mocked::poll_platform::MockPlatform;
use crate::observers::mocked::polling::{HistoricPollState, MockPoller};
use crate::observers::mocked::record_platform::MockRecordPlatform;
use crate::testing::{norm, Lambda};
use crate::value::Value;

pub struct InstantWriter {
    do_at: Option<Tick>,
    to_write: Option<Value>
}

impl InstantWriter {

    pub fn new() -> Self {
        InstantWriter { do_at: None, to_write: None }
    }
    pub fn do_write(&mut self, value: Value, poller: &mut MockPoller, now: &Tick) {
        if now >= &poller.current.send_at { // Poll has been sent - schedule write for when it replies
            self.do_at = Some(poller.current.reply_at); // Write in the same INSTANT which it replies.
        } else { // Poll not yet sent! Send now.
            self.do_at = Some(now.clone());
        }
        self.to_write = Some(value);
    }

    pub fn do_tick(&mut self, platform: &mut MockPlatform, observer: &mut MockPoller, now: &Tick) {
        if self.do_at.as_ref().is_some_and(|x| x == now){
            platform.value = self.to_write.unwrap();
            observer.last = Some(HistoricPollState {
                sent: now.clone(),
                process: now.clone(),
                value: self.to_write.unwrap(),
                replied: now.clone(),
            });
            self.do_at = None;
            self.to_write = None;
        }
    }
}

pub fn instant_write( observer: &mut MockPoller, value: Value, now: &Tick) {

}


pub struct LossyWriter {
    to_write: Option<Value>,
    send_at: Option<Tick>,
    process_at: Option<Tick>,
    reply_at: Option<Tick>,
    rtt_lambda: Lambda,
    rtt_std_dev: Lambda,
}

impl LossyWriter {
    pub fn new(rtt_lambda: Lambda, rtt_std_dev: Lambda) -> Self {
        LossyWriter {
            send_at: None,
            to_write: None,
            process_at: None,
            reply_at: None,
            rtt_lambda,
            rtt_std_dev
        }
    }

    pub fn do_write(&mut self, value: Value, poller: &mut MockPoller, now: &Tick) {
        self.to_write = Some(value);


        if now >= &poller.current.send_at { // Poll has been sent - schedule write for when it replies
            let send_at = poller.current.reply_at;
            let process_at = send_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());
            let reply_at = process_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());

            // TODO: ASSUMING BACKOFF GREATER THAN WRITE TIME.
            // TODO: Therefore - platform's next poll will not conflict if this is scheduled after the current replies.
            // Modifying poll state to reschedule next adds unnecessary complexity.

            self.send_at = Some(send_at);
            self.process_at = Some(process_at);
            self.reply_at = Some(reply_at);
            self.to_write = Some(value);
        } else { // Poll not yet sent! Send now and reschedule poll for after.
            let send_at = now;
            let process_at = send_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());
            let reply_at = process_at + norm(self.rtt_lambda/2.0, self.rtt_std_dev, &mut rng());

            self.reply_at = Some(reply_at);
            self.process_at = Some(process_at);
            self.reply_at = Some(reply_at);
            self.to_write = Some(value);

            // Reschedule next poll for AFTER write.
            poller.current.send_at = self.reply_at.unwrap() + 1u64; // Add one, since this runs AFTER poll.
            poller.current.process_at = poller.current.send_at + norm(poller.rtt_lambda/2.0, poller.rtt_std_dev, &mut rng());
            poller.current.reply_at = poller.current.process_at + norm(poller.rtt_lambda/2.0, poller.rtt_std_dev, &mut rng());

            // Give poller written value as LAST!
            poller.last = Some(HistoricPollState {
                sent: send_at.clone(),
                process: 0, // TODO: Does this get used?
                value,
                replied: reply_at.clone(), // Poller would know reply time ASSUMING BACKOFF > WRITE TIME
            })
        }

    }

    pub fn do_tick(&mut self, platform: &mut MockPlatform, now: &Tick) {
        if self.process_at.as_ref().is_some_and(|x|x == now)  {
            platform.value = self.to_write.unwrap();
            self.process_at = None;
            self.to_write = None;
            self.send_at = None;
            self.reply_at = None;
        }
    }
}