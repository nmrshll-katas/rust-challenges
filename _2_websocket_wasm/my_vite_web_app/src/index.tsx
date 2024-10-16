/* @refresh reload */
import { render } from "solid-js/web";
import { Component, createSignal } from "solid-js";
// @ts-ignore
import init from "my-wasm-web?init";
import { WS_URL, wsPing } from "./ws_ping";

const App: Component = () => {
  const [respTxt, setRespTxt] = createSignal<string | null>(null);

  async function handle_websocket() {
    await init();
    let got_from_rust = await wsPing(WS_URL, "Hello, world!");
    console.log({ got_from_rust });
    setRespTxt(got_from_rust);
  }

  return (
    <div>
      <header>
        <p> my app !</p>
        <button onClick={handle_websocket} style="margin: 10px">
          Ping websocket
        </button>
        {respTxt() && (
          <div style="margin: 10px;">
            Got response from websocket:
            <div style="margin: 10px; background-color: #f0f0f0; padding: 10px; border-radius: 5px;">{respTxt()}</div>
          </div>
        )}
      </header>
    </div>
  );
};

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error("Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?");
}

render(() => <App />, root!);
