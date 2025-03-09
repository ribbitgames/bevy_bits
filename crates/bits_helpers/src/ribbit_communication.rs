use std::sync::{Arc, LazyLock};

use bevy::prelude::*;
use parking_lot::Mutex;
use ribbit_bits::{BitDuration, BitMessage, BitParameters, BitResult, RibbitMessage};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::MessageEvent;

pub static RIBBIT_MESSAGE_QUEUE: LazyLock<Arc<Mutex<Vec<RibbitMessage>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[cfg(not(target_arch = "wasm32"))]
pub static BIT_MESSAGE_QUEUE: LazyLock<Arc<Mutex<Vec<BitMessage>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[cfg(target_arch = "wasm32")]
pub fn listen_ribbit_messages() {
    let window = web_sys::window().expect("no global `window` exists");
    let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
        let message: Result<RibbitMessage, serde_wasm_bindgen::Error> =
            serde_wasm_bindgen::from_value(event.data());

        let Ok(message) = message else {
            error!("Could not parse ribbit message {:?}", &event.data());
            return;
        };

        RIBBIT_MESSAGE_QUEUE.lock().push(message);
    }) as Box<dyn FnMut(MessageEvent)>);

    window
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .expect("failed to add message event listener");

    closure.forget(); // Leaks memory, but ensures the closure lives for the lifetime of the program
}

#[cfg(not(target_arch = "wasm32"))]
pub fn send_bit_message(message: BitMessage) {
    BIT_MESSAGE_QUEUE.lock().push(message);
}

#[cfg(target_arch = "wasm32")]
pub fn send_bit_message(message: BitMessage) {
    let window = web_sys::window().expect("no global `window` exists");
    let Ok(message_str) = serde_wasm_bindgen::to_value(&message) else {
        error!("Could not serialize {message:?}");
        return;
    };

    let Ok(Some(parent_window)) = window.parent() else {
        error!("{message:?} not sent, parent_window not found.");
        return;
    };

    if let Err(err) = parent_window.post_message(&message_str, "*") {
        error!("Could not post message {message_str:?}. {err:?}");
    };
}

/// This trait implements the messages that can be called by Ribbit.
///
/// The functions needs to be implemented for the good functionning of the platform.
/// Those functions are not meant to be called directly from the bit itself.
pub trait RibbitMessageHandler: Send + Sync + Default + 'static {
    fn duration(world: &mut World) -> BitDuration;
    fn end(world: &mut World) -> BitResult;
    fn restart(world: &mut World);
}

fn proccess_ribbit_messages<T: RibbitMessageHandler>(world: &mut World) {
    let messages = RIBBIT_MESSAGE_QUEUE.lock().drain(..).collect::<Vec<_>>();

    for message in messages {
        match message {
            RibbitMessage::End => {
                let result = T::end(world);
                send_bit_message(BitMessage::End(result));
            }
            RibbitMessage::Parameters => {
                let duration = T::duration(world);
                let parameters = BitParameters { duration };
                send_bit_message(BitMessage::Parameters(parameters));
            }
            RibbitMessage::Restart => T::restart(world),
            RibbitMessage::Start => {
                // Todo : Block in PostStartup until we receive this message.
            }
        }
    }
}

fn ready() {
    send_bit_message(BitMessage::Ready);
}

#[derive(Default)]
pub struct RibbitCommunicationPlugin<T: RibbitMessageHandler>(core::marker::PhantomData<T>);

impl<T: RibbitMessageHandler> Plugin for RibbitCommunicationPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, proccess_ribbit_messages::<T>);
        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(Startup, listen_ribbit_messages);
        }
        app.add_systems(PostStartup, ready);
    }
}
