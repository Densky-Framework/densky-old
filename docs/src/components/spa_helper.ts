type Listener = () => void;

interface Window {
  spa_router_listeners: Array<Listener>;
  on_router_change(listener: Listener): void;
}

window.spa_router_listeners ??= [];

window.on_router_change = (listener) => window.spa_router_listeners.push(listener);

function notifyListeners() {
  window.spa_router_listeners.forEach((listener) => listener());
}

// Detect route change in dev-mode spa
const actualTitle = document.querySelector(".markdown > h1")?.innerHTML;
for (const aElement of document.getElementsByTagName("a")) {
  aElement.addEventListener("click", () => {
    const retry = () => {
      const newTitle = document.querySelector(".markdown > h1")?.innerHTML;

      if (actualTitle !== newTitle) return notifyListeners();

      setTimeout(retry, 100);
    };

    retry();
  });
}
