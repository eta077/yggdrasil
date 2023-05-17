import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function onFindFitsClicked(event) {
  fetch("/earendel/apod-fits");
}

function App(props) {
  let blob = new Blob([new Uint8Array(props.state.img).buffer], {type: "image/png"});
  let img_url = URL.createObjectURL(blob);
  let copyright_text = "";
  if (props.state.copyright) {
    copyright_text = "© " + props.state.copyright;
  }
  return html`
    <div class="container-fluid">
      <div class="row align-items-center">
        <div class="col-4">
          <div class="row">
            <img class="img-fluid" src=${img_url}></img>
          </div>
          <div class="row">
            <label>${props.state.title}</label>
          </div>
          <div class="row">
            <label>${copyright_text}</label>
          </div>
        </div>
        <div class="col-4">
          <button class="btn btn-primary" onClick=${(event) => onFindFitsClicked(event)}>Find FITS files</button>
        </div>
      </div>
    </div>
  `;
}

let response = await fetch("/earendel/apod");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} state=${json}></${App}>`, document.body);
}

