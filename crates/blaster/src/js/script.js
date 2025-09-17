const root = document.getElementById("app");
const ws = new WebSocket("http://127.0.0.1:8080/__ws");
ws.addEventListener("message", (e) => {
  {
    const patch = JSON.parse(e.data);
    switch (patch["type"]) {
      case "ReplaceInner":
        const element = document.querySelector(patch["selector"]);
        cleanupEventListeners(element);
        element.innerHTML = patch["html"];
        setupEventListeners(element);
        break;
    }
  }
});
function setupEventListeners(element) {
  element.querySelectorAll("[data-id]").forEach((element) => {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.addEventListener(eventType, handleEvent),
    );
  });
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
  element.querySelectorAll("[data-id]").forEach((element) => {
    const events = element.dataset.events.split(",");
    events.forEach((eventType) =>
      element.removeEventListener(eventType, handleEvent),
    );
  });
}

class View extends HTMLElement { }
customElements.define("bv-view", View);
