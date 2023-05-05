import {h, render} from "https://unpkg.com/preact?module";
import htm from "https://unpkg.com/htm?module";

const html = htm.bind(h);

let selectedLesson = null;
let draggedPart = null;

function onLessonClick(lessonInfo) {
  let target = document.getElementById("container_" + lessonInfo.name);
  if (selectedLesson) {
    let lesson = selectedLesson.children.namedItem("lesson");
    lesson.classList.remove("border-3");
    lesson.classList.add("border-1");
    let parts = selectedLesson.children.namedItem("parts");
    parts.classList.remove("border");
    parts.classList.remove("border-3");
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
  parts.classList.add("border");
  parts.classList.add("border-3");
  render(html`<${LessonPartsNode} parts=${lessonInfo.available_parts}></${LessonPartsNode}`, parts);
}

function onPartDragStart(event) {
  draggedPart = event.target;
  draggedPart.classList.add("dragging");
}

function onPartDragEnd(event) {
  console.log("onPartDragEnd");
  event.target.classList.remove("dragging");
}

function onLessonDragOver(event) {
  event.preventDefault();
}

function onLessonDragEnter(event) {
  
}

function onLessonDrop(event, lessonInfo) {
  console.log("onLessonDrop");
  event.preventDefault();

  let changing_part = null;
  let avail_parts = lessonInfo.available_parts;
  for (var i = 0; i < avail_parts.length; i++) {
    if (avail_parts[i].name == draggedPart.innerHTML) {
      changing_part = avail_parts.splice(i, 1)[0];
      break;
    }
  }
  lessonInfo.current_parts.push(changing_part);

  fetch("/kids/set-lesson", {
    method: "POST",
    body: JSON.stringify(lessonInfo),
    headers: {
      "Content-type": "application/json; charset=utf-8"
    }
  });

  event.target.appendChild(draggedPart);
}

function onPartsDragOver(event) {
  event.preventDefault();
}

function onPartsDragEnter(event) {
  
}

function onPartsDrop(event, lessonInfo) {
  event.preventDefault();

  let changing_part = null;
  let cur_parts = lessonInfo.current_parts;
  for (var i = 0; i < cur_parts.length; i++) {
    if (cur_parts[i].name == draggedPart.innerHTML) {
      changing_part = cur_parts.splice(i, 1)[0];
      break;
    }
  }
  lessonInfo.available_parts.push(changing_part);

  fetch("/kids/set-lesson", {
    method: "POST",
    body: JSON.stringify(lessonInfo),
    headers: {
      "Content-type": "application/json; charset=utf-8"
    }
  });

  let target = document.getElementById("parts_" + lessonInfo.name);
  target.appendChild(draggedPart);
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
      <div id="parts_${lesson.name}" name="parts" class="col-3 py-3" onDragOver="${(event) => onPartsDragOver(event)}" onDragEnter="${(event) => onPartsDragEnter(event)}" onDrop="${(event) => onPartsDrop(event, lesson)}">
      </div>
      <div id="lesson_${lesson.name}" name="lesson" class="col-9 border border-primary border-1" onClick="${(event) => onLessonClick(lesson)}" onDragOver="${(event) => onLessonDragOver(event)}" onDragEnter="${(event) => onLessonDragEnter(event)}" onDrop="${(event) => onLessonDrop(event, lesson)}">
        ${lesson.name}
      </div>
    </div>
  `;
}

function LessonPartsNode(parts) {
  return html`
    ${parts.parts.map((part) => {
      return html`
        <p draggable="true" class="border border-light" onDragStart="${(event) => onPartDragStart(event)}" onDragEnd="${(event) => onPartDragEnd(event)}">${part.name}</p>
      `;
    })}
  `;
}


let response = await fetch("/kids/lessons");
if (response.status == 200) {
  let json = await response.json();
  render(html`<${App} lessons=${json}></${App}>`, document.body);
}

