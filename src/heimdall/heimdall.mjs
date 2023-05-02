import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function Schematic(props) {
  return html`
    <div class="container">
      ${props.devices.map((device) => {
        return DeviceNode(device);
      })}
    </div>
  `;
}

function DeviceNode(device) {
  return html`
    <div class="container">
      <div class="row">
        <div class="col">${device.name}</div>
        <div class="col">CPU: ${device.cpu_usage}%</div>
      </div>
      <div class="row">
        <div class="col"></div>
        <div class="col">Mem: ${device.mem_usage}%</div>
      </div>
    </div>  
  `;
}

let url = new URL("/ws/heimdall", window.location.href);
url.protocol = url.protocol.replace("http", "ws");
let ws = new WebSocket(url.href);
ws.onmessage = (event) => {
  let json = JSON.parse(event.data);
  render(html`<${Schematic} devices=${json}></${Schematic}>`, document.body);
}

