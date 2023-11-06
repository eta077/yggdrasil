import { h, render } from "https://esm.sh/preact@5.3.1";
import htm from "https://esm.sh/htm@3.1.1";

const html = htm.bind(h);

async function onFindFitsClicked(event) {
  console.log("event: " + event);
  
  let fits_div = document.getElementById("fits-div");
  let fits_btn = fits_div.children.namedItem("find-fits-btn");
  fits_btn.classList.add("disabled");
  let fits_lbl = fits_div.children.namedItem("find-fits-lbl");
  fits_lbl.innerHTML = "Loading FITS list...";

  let response = await fetch("/earendel/apod-fits");
  if (!response.ok) {
    console.log("FITS response was error: " + response);
    fits_lbl.innerHTML = "An error occurred while finding FITS files.";
    fits_btn.classList.remove("disabled");
    return;  
  }
  let json = await response.json();
  console.log("response: " + json);
  fits_div.innerHTML = "";
  render(html`<${FitsList} state=${json}></${FitsList}>`, fits_div);
}

function App(props) {
  let blob = new Blob([new Uint8Array(props.state.img).buffer], { type: "image/png" });
  let img_url = URL.createObjectURL(blob);
  let copyright_text = "";
  if (props.state.copyright) {
    copyright_text = "© " + props.state.copyright;
  }
  return html`
    <div class="container-fluid">
      <div class="row align-items-center">
        <div class="col">
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
        <div id="fits-div" class="col">
          <button id="find-fits-btn" class="btn btn-primary" onClick=${(event) => onFindFitsClicked(event)}>Find FITS files</button>
          <p id="find-fits-lbl"></p>
        </div>
      </div>
    </div>
  `;
}

function FitsList(props) {
  return html`
    <div class="list-group">
      ${props.state.files.map((file) => {
        return html`<button id=${file} class="list-group-item">${file}</button>`;
      })}
    </div>
  `;
}

render(html`<p>Loading...</p>`, document.body);
let response = await fetch("/earendel/apod");
if (response.ok) {
  let json = await response.json();
  document.body.innerHTML = "";
  render(html`<${App} state=${json}></${App}>`, document.body);
}

