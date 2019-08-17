use std::sync::Arc;
use crate::view::View;
use crate::tree::ViewTree;
use crossbeam::channel::TryRecvError;
use crossbeam::{channel, Receiver, Sender};
use std::process::exit;

#[cfg(target_os = "macos")]
use swift_birb::protocol;

type EventSender = Sender<protocol::SBEvent>;

/// Connects a view tree to the native backend.
pub struct Host {
    pub tree: ViewTree,
    event_recv: Receiver<protocol::SBEvent>,

    #[cfg(target_os = "macos")]
    native: swift_birb::Host,
}

impl Host {
    /// Creates a new Host.
    ///
    /// The newly created tree will be initialized, but it won’t be rendered until you call `poll`.
    pub fn new(root: Arc<dyn View>) -> Host {
        let (event_sender, event_recv) = channel::unbounded();

        Host {
            tree: ViewTree::new(root),
            event_recv,

            #[cfg(target_os = "macos")]
            native: unsafe {
                let native_user_data =
                    Box::into_raw(Box::new(event_sender)) as *const EventSender as usize;

                swift_birb::Host::new(
                    Some(raw_event_handler),
                    native_user_data,
                )
            },
        }
    }

    /// Receives all events from the event queue and updates the tree accordingly.
    pub fn poll(&mut self) {
        loop {
            match self.event_recv.try_recv() {
                Ok(event) => self.recv_raw_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("event receiver has been disconnected"),
            }
        }
    }

    fn recv_raw_event(&mut self, _event: protocol::SBEvent) {
        unimplemented!("receive raw event")
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let sender_ptr = unsafe { self.native.user_data() } as *mut EventSender;
            let sender = unsafe { Box::from_raw(sender_ptr) };
            drop(sender);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn raw_event_handler(event: protocol::SBEvent, user_data: usize) {
    let sender_ptr = user_data as *mut EventSender;
    let sender = &*sender_ptr;
    if let Err(err) = sender.send(event) {
        eprintln!("Failed to send raw event: {}", err);
        exit(1);
    }
}
