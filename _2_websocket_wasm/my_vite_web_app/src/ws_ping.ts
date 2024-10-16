// @ts-ignore
import init, * as wasm from "my-wasm-web?init";

export const WS_URL = "wss://echo.websocket.events";

export function wsPing(endpoint: string, message: string): Promise<string> {
  return wasm.ws_ping(endpoint, message);
}

export function wsPing_ts(endpoint: string, message: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(endpoint);
    socket.onopen = () => {
      socket.send(message);
    };
    socket.onmessage = (event) => {
      resolve(event.data);
      socket.close();
    };
    socket.onerror = (error) => {
      reject(error);
      socket.close();
    };
  });
}
