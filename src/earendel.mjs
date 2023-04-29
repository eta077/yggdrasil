import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function App(props) {
  let blob = new Blob([new Uint8Array(props.state.img).buffer], {type: "image/png"});
  let img_url = URL.createObjectURL(blob);
  return html`
    <div class="container">
      <div class="row">
        <img class="img-fluid" src=${img_url}></img>
      </div>
      <div class="row">
        <label>${props.state.title}</label>
      </div>
    </div>
  `;
}

let response = await fetch("/earendel/apod");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} state=${json}></${App}>`, document.body);
}

