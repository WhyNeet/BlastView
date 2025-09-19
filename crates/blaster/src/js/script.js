const root = document.getElementById("app");
const ws = new WebSocket("http://127.0.0.1:8080/__ws/$SESSION_ID");
ws.addEventListener("open", () => setupEventListeners(document));
ws.addEventListener("message", (e) => {
  {
    const patch = JSON.parse(e.data);
    applyPatch(patch);
  }
});

function applyPatch(patch) {
  switch (patch["type"]) {
    case "ReplaceInner":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          cleanupEventListeners(element);
          element.innerHTML = patch["html"];
          setupEventListeners(element);
        });
      }
      break;
    case "ReplaceOuter":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          cleanupEventListeners(element);
          element.outerHTML = patch["html"];
          setupEventListeners(element);
        });
      }
      break;
    case "SetAttribute":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          element.setAttribute(patch["name"], patch["value"]);
        });
      }
      break;
    case "RemoveAttribute":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          element.removeAttribute(patch["name"]);
        });
      }
      break;
    case "ReplaceChild":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          console.log(element);
          const oldChild = element.childNodes[patch["index"]];
          cleanupEventListeners(oldChild);
          const div = document.createElement("div");
          div.innerHTML = patch["html"];
          const newChild = div.firstChild;
          element.replaceChild(newChild, oldChild);
          setupEventListeners(newChild);
        });
      }
      break;
    case "RemoveElement":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          cleanupEventListeners(element);
          element.remove();
        });
      }
      break;
    case "AttachEvent":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          if (element.dataset.events) {
            element.dataset.events = element.dataset.events + "," + patch["event"];
          } else {
            element.dataset.events = patch["event"];
          }
          element.addEventListener(patch["event"], handleEvent);
        });
      }
      break;
    case "DetachEvent":
      {
        const elements = document.querySelectorAll(patch["selector"]);
        elements.forEach((element) => {
          if (element.dataset.events) {
            const events = element.dataset.events.split(",");
            element.dataset.events = events.filter(event => event !== patch["event"]);
          }
          element.removeEventListener(patch["event"], handleEvent);
        });
      }
      break;
    case "Batch":
      patch["patches"].forEach(applyPatch);
  }
}

function setupEventListeners(element) {
  element.querySelectorAll?.("[data-events]")?.forEach((element) => {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.addEventListener(eventType, handleEvent),
    );
  });
  if (element.dataset?.events) {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.addEventListener(eventType, handleEvent),
    );
  }
}

function handleEvent(e) {
  const dataId = e.target.getAttribute("data-id");
  const eventType = e.type;
  const eventId = `${dataId}_${eventType}`;

  if (ws.readyState === WebSocket.OPEN) {
    ws.send(eventId);
  }
}

function cleanupEventListeners(element) {
  element.querySelectorAll?.("[data-events]")?.forEach((element) => {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.removeEventListener(eventType, handleEvent),
    );
  });
  if (element.dataset?.events) {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.removeEventListener(eventType, handleEvent),
    );
  }
}

class View extends HTMLElement { }
customElements.define("bv-view", View);
