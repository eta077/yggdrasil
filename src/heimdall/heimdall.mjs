import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function Schematic(props) {
  return html`
    <div class="container-fluid">
      <div class="row">
        <div class="col">
          ${props.devices.filter((device) => device.connection == "Wireless").map((device) => {
            return DeviceNode(device);
          })}
        </div>
        <div class="col">
          ${props.devices.filter((device) => device.connection == "Origin").map((device) => {
            return DeviceNode(device);
          })}
        </div>
        <div class="col">
          ${props.devices.filter((device) => device.connection == "Wired").map((device) => {
            return DeviceNode(device);
          })}
        </div>
      </div>
    </div>
  `;
}

function DeviceNode(device) {
  let img_url = window.location.href + "/assets/" + device.connection + ".png";
  let popup_title = "";
  let popup_content = "";
  if (device.capabilities.length > 0) {
    popup_title = device.name + " Actions";
  }
  if (device.cpu_usage > 0.0 || device.mem_usage > 0.0) {
    popup_title = device.name + " Actions";
    popup_content += "<div class=\"container\">"
      + "<div class=\"row\">"
        + "<div class=\"col\">"
          + "<pre>CPU: </pre>"
        + "</div>"
        + "<div class=\"col\">" 
          + "<pre>" + device.cpu_usage + "</pre>"
        + "</div>"
      + "</div>"
      + "<div class=\"row\">"
        + "<div class=\"col\">"
          + "<pre>Mem: </pre>"
        + "</div>"
        + "<div class=\"col\">"
          + "<pre>" + device.cpu_usage + "</pre>"
      + " </div> </div>";
  }
  return html`
    <div class="container text-center" data-bs-toggle="popover" data-bs-title="${popup_title}" data-bs-content="${popup_content}">
        <img class="img-fluid" src=${img_url}></img>
        <pre>${device.name}</pre>
    </div>
  `;
}

let rendered = false;
let url = new URL("/ws/heimdall", window.location.href);
url.protocol = url.protocol.replace("http", "ws");
let ws = new WebSocket(url.href);
ws.onmessage = (event) => {
  if (!rendered) {
    let json = JSON.parse(event.data);
    render(html`<${Schematic} devices=${json}></${Schematic}>`, document.body);
    rendered = true;
    const popoverTriggerList = document.querySelectorAll('[data-bs-toggle="popover"]');
    [...popoverTriggerList].map(popoverTriggerEl => new bootstrap.Popover(popoverTriggerEl, {html: true}));
  }
}

