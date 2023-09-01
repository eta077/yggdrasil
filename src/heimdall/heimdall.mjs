import { h, render } from "https://esm.sh/preact@5.3.1";
import htm from "https://esm.sh/htm@3.1.1";

const html = htm.bind(h);

function Schematic(props) {
  return html`
    <div id="schematic" class="container-fluid">
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
          + "<pre id=" + device.name + "-cpu-val>" + device.cpu_usage + "</pre>"
        + "</div>"
      + "</div>"
      + "<div class=\"row\">"
        + "<div class=\"col\">"
          + "<pre>Mem: </pre>"
        + "</div>"
        + "<div class=\"col\">"
          + "<pre id=" + device.name + "-mem-val>" + device.mem_usage + "</pre>"
      + " </div> </div>";
  }
  return html`
    <div class="container text-center" data-bs-toggle="popover" title=${popup_title} data-bs-content=${popup_content}>
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
  let json = JSON.parse(event.data);
  if (rendered) {
    let target = document.getElementById("schematic");
    render(html`<${Schematic} devices=${json}></${Schematic}>`, document.body, target);
    
    for (let device of json) {
      let cpu_label = document.getElementById(device.name + "-cpu-val");
      if (cpu_label != null) {
        cpu_label.innerHTML = device.cpu_usage;
      }
      let mem_label = document.getElementById(device.name + "-mem-val");
      if (mem_label != null) {
        mem_label.innerHTML = device.mem_usage;
      }
    }
  } else {
    render(html`<${Schematic} devices=${json}></${Schematic}>`, document.body);
    const popoverTriggerList = document.querySelectorAll('[data-bs-toggle="popover"]');
    [...popoverTriggerList].map(popoverTriggerEl => {
      bootstrap.Popover.getOrCreateInstance(popoverTriggerEl, { html: true });
    });
    rendered = true;
  }
}
