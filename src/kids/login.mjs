import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

async function submitLogin(event) {
  console.log(event);
  let response = await fetch(event.target.action, {
    method: event.target.method,
    body: new URLSearchParams(new FormData(event.target))
  });
  console.log(response);
  if (response.ok) {
    window.location.href = response.url;
  } else {
    let text = await response.text();
    render(html`<${App} error=${text}></${App}>`, document.body);
  }
}

function App(props) {
  let error = "";
  if (props && props.error) {
    error = props.error;
  }
  return html`
    <form id="loginForm" action="/kids/login" method="post" class="container">
      <div class="row my-3">
        <label for="emailInput" class="form-label">Email address</label>
        <input type="email" id="emailInput" name="username" class="form-control"></input>
      </div>
      <div class="row my-3">
        <label for="passwordInput" class="form-label">Password</label>
        <input type="password" id="passwordInput" name="password" class="form-control"></input>
      </div>
      <div class="row my-3 row-cols-auto">
        <button type="submit" class="btn btn-primary">Login</button>
      </div>
      <div class="row my-3">
        <p class="text-danger">${error}</p>
      </div>
    </form>
  `;
}

render(html`<${App}></${App}>`, document.body);

window.addEventListener("load", () => {
  const form = document.getElementById("loginForm");
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    submitLogin(event);
  })
});



