(() => {
  const tabTraps = document.getElementsByClassName(
    "tabtrap",
  ) as HTMLCollectionOf<HTMLElement>;
  for (const tabTrap of tabTraps) {
    const onFocus = (dir: number) => {
      const items = [...tabTrap.querySelectorAll("[tabindex]")] as Array<
        HTMLElement
      >;
      const focused = document.activeElement as HTMLElement;

      const actualIndex = focused ? items.indexOf(focused) : -1;
      let newIndex = actualIndex + dir;
      newIndex = newIndex < 0
        ? items.length + newIndex
        : newIndex % items.length;

      const newFocusElement = items[newIndex];
      newFocusElement.focus();
    };
    tabTrap.addEventListener("keydown", (event) => {
      if (event.key === "Tab") {
        onFocus(event.shiftKey ? -1 : 1);
        event.preventDefault();
      }
    });
    tabTrap.addEventListener("focusin", (event) => {
      if (event.currentTarget === event.target) {
        onFocus(1);
        event.preventDefault();
      }
    });
  }
})();
