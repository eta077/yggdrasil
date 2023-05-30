import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function App(props) {
  let content = { __html: props.text };
  return html`
    <div class="container-fluid px-5" dangerouslySetInnerHTML=${content}></div>
  `;
}

let response = await fetch("/blog/content");
if (response.status == 200) {
  let text = await response.text();
  render(html`<${App} text=${text}></${App}>`, document.body);
}
