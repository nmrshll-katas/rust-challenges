use js_sys::Promise;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
macro_rules! console_err {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

}

#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}

#[wasm_bindgen]
// exporting async functions to wasm needs crate wasm_bindgen_futures
pub async fn ws_ping(endpoint: &str, message: &str) -> Promise {
    let prom = Promise::new(&mut move |resolve, reject| {
        console_log!("Connecting to {}", endpoint);
        let ws = WebSocket::new(endpoint).expect_throw("Failed creating WebSocket");

        // Callback for message received
        let onmessage_callback_box: Box<dyn FnMut(MessageEvent)> =
            Box::new(move |evt: MessageEvent| {
                console_log!("onMessageCallback: {:?}", evt.data());

                let txt = match evt.data().dyn_into::<js_sys::JsString>() {
                    Ok(txt) => {
                        console_log!("message event, received Text: {:?}", txt);
                        txt
                    }
                    Err(other) => Err(other).expect_throw("message event: received unknown type"),
                };
                resolve.call1(&JsValue::NULL, &txt).unwrap();
            });
        let onmessage_callback = Closure::wrap(onmessage_callback_box);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Error callback
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |ev: ErrorEvent| {
            console_err!("error event: {:?}", ev);
            reject
                .call1(&JsValue::NULL, &JsValue::from(ev.message()))
                .unwrap();
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget(); // forget closure to avoid garbage-collection: will keep callback alive

        // open/init callback
        let cloned_ws = ws.clone();
        let cloned_msg = message.to_string();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log!("socket opened");
            match cloned_ws.send_with_str(&cloned_msg) {
                Ok(_) => console_log!("message successfully sent: {:?}", cloned_msg),
                Err(err) => console_log!("error sending message: {:?}", err),
            }
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    });

    prom
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
