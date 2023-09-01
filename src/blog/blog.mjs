import { h, render } from "https://esm.sh/preact@5.3.1";
import htm from "https://esm.sh/htm@3.1.1";

const html = htm.bind(h);

async function onNavClicked(event, target) {
  let response = await fetch("/blog/content?selection=" + target);
  if (response.status == 200) {
    let json = await response.json();
    render(html`<${App} state=${json}></${App}>`, document.body);
  }
}

function App(props) {
  let content = { __html: props.state.markup };
  let previous_target = props.state.previous;
  let next_target = props.state.next;
  return html`
    <ul class="nav nav-fill">
      <li class="nav-item">
        <button class="btn btn-link" onClick=${(event) => onNavClicked(event, previous_target)}>Previous</button>
      </li>
      <li class="nav-item">
        <button class="btn btn-link" onClick=${(event) => onNavClicked(event, next_target)}>Next</button>
      </li>
    </ul>
    <div class="container-fluid px-5" dangerouslySetInnerHTML=${content}></div>
  `;
}

let response = await fetch("/blog/content");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} state=${json}></${App}>`, document.body);
}
