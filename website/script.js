const scenarioTabs = [...document.querySelectorAll('.scenario-tablist [role="tab"]')];

function selectScenario(tab, moveFocus = false) {
  scenarioTabs.forEach((item) => {
    const isSelected = item === tab;
    item.setAttribute('aria-selected', String(isSelected));
    item.tabIndex = isSelected ? 0 : -1;
    const panel = document.getElementById(item.getAttribute('aria-controls'));
    panel.hidden = !isSelected;
    const video = panel.querySelector('video');
    if (video) {
      if (isSelected) video.play().catch(() => {});
      else {
        video.pause();
        video.currentTime = 0;
      }
    }
  });
  if (moveFocus) tab.focus();
}

scenarioTabs.forEach((tab, index) => {
  tab.addEventListener('click', () => selectScenario(tab));
  tab.addEventListener('keydown', (event) => {
    let nextIndex;
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') nextIndex = (index + 1) % scenarioTabs.length;
    if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') nextIndex = (index - 1 + scenarioTabs.length) % scenarioTabs.length;
    if (event.key === 'Home') nextIndex = 0;
    if (event.key === 'End') nextIndex = scenarioTabs.length - 1;
    if (nextIndex !== undefined) {
      event.preventDefault();
      selectScenario(scenarioTabs[nextIndex], true);
    }
  });
});

const selectedScenario = scenarioTabs.find((tab) => tab.getAttribute('aria-selected') === 'true');
if (selectedScenario) selectScenario(selectedScenario);
