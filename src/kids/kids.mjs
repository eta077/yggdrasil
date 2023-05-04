import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

function App(props) {
  return html`
    <div class="container-fluid">
      ${props.lessons.map((lesson) => {
        return LessonNode(lesson);
      })}
    </div>
  `;
}

function LessonNode(lesson) {
  return html`
    <div id="container_${lesson.name}" class="row">
      <div id="parts_${lesson.name}" name="parts" class="col-3 invisible">
        <${LessonPartsNode} parts=${lesson.available_parts}></${LessonPartsNode}>
      </div>
      <div id="lesson_${lesson.name}" name="lesson" class="col-9 border border-primary border-1">
        ${lesson.name}
      </div>
    </div>
  `;
}

function LessonPartsNode(parts) {
  return html`
    ${parts.parts.map((part) => {
      return html`
        ${part.name}
      `;
    })}
  `;
}

let selectedLesson = null;

let response = await fetch("/kids/lessons");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} lessons=${json}></${App}>`, document.body);
}

window.addEventListener("click", (event) => {
  if (event.target.id && event.target.id.startsWith("lesson_")) {
    let target = event.target.parentElement;
    if (selectedLesson) {
        let lesson = selectedLesson.children.namedItem("lesson");
        lesson.classList.remove("border-3");
        lesson.classList.add("border-1");
        let parts = selectedLesson.children.namedItem("parts");
        parts.classList.remove("visible");
        parts.classList.add("invisible");
      if (selectedLesson.id == target.id) {
        selectedLesson = null;
        return;
      }
    }
    selectedLesson = target;
    let lesson = selectedLesson.children.namedItem("lesson");
    lesson.classList.remove("border-1");
    lesson.classList.add("border-3");
    let parts = selectedLesson.children.namedItem("parts");
    parts.classList.remove("invisible");
    parts.classList.add("visible");
  }
});
