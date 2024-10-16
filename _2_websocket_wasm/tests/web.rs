//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

use _2_websocket_wasm::ws_ping2;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);
const ENDPOINT: &str = "wss://echo.websocket.events";

#[wasm_bindgen_test]
async fn pass() -> Result<(), Box<dyn std::error::Error>> {
    let promise = ws_ping2(ENDPOINT, "hello ws").await;
    let output = JsFuture::from(promise).await.unwrap();

    assert_eq!(
        output,
        JsValue::from_str("echo.websocket.events sponsored by Lob.com")
    );

    Ok(())
}
