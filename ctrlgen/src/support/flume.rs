use crate::Proxy;

/// A proxy that sends Msg through a [::flume] channel
pub struct FlumeProxy<Msg> {
    sender: flume::Sender<Msg>,
}

impl<Msg> FlumeProxy<Msg> {
    pub fn new(sender: flume::Sender<Msg>) -> Self {
        Self { sender }
    }
}

impl<Msg: core::fmt::Debug> Proxy<Msg> for FlumeProxy<Msg> {
    fn send(&self, msg: Msg) {
        self.sender.send(msg).unwrap()
    }
}
