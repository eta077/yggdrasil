import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

let selectedLesson = null;

function onLessonClick(lessonInfo) {
  let target = document.getElementById("container_" + lessonInfo.name);
  if (selectedLesson) {
    let lesson = selectedLesson.children.namedItem("lesson");
    lesson.classList.remove("border-3");
    lesson.classList.add("border-1");
    let parts = selectedLesson.children.namedItem("parts");
    parts.innerHTML = "";
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
  parts.innerHTML = LessonPartsNode(lessonInfo.available_parts);
}

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
      <div id="parts_${lesson.name}" name="parts" class="col-3">
      </div>
      <div id="lesson_${lesson.name}" name="lesson" class="col-9 border border-primary border-1" onClick="${(event) => onLessonClick(lesson)}">
        ${lesson.name}
      </div>
    </div>
  `;
}

function LessonPartsNode(parts) {
  return html`
    ${parts.map((part) => {
      return html`
        ${part.name}
      `;
    })}
  `;
}


let response = await fetch("/kids/lessons");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} lessons=${json}></${App}>`, document.body);
}

