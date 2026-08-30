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

/* 다운로드 버튼 — 접속 환경을 감지해 하나를 강조한다.
 *
 * 강조만 하고 숨기지 않는다: 감지는 빗나갈 수 있고(특히 아래 Mac 아키텍처),
 * JS 가 없는 환경도 있다. 셋 다 보이면 최악의 경우에도 사용자가 직접 고르면 된다.
 *
 * ⚠️ Mac 아키텍처는 UA 로 알 수 없다 — Apple Silicon 의 Safari 도 "MacIntel" 을 보고한다.
 * 그래서 기본은 Apple Silicon(현행 모델 전부)으로 두고, Chromium 계열에서만 제공되는
 * userAgentData 로 x86 이 확인될 때 Intel 로 바꾼다. 네트워크 호출은 없다(CSP connect-src 'none'). */
(function () {
  const buttons = [...document.querySelectorAll('.dl-button[data-platform]')];
  if (!buttons.length) return;

  const suggest = (id) => {
    buttons.forEach((b) => b.classList.toggle('is-suggested', b.dataset.platform === id));
  };

  const uaData = navigator.userAgentData;
  const platform = (uaData && uaData.platform) || '';
  const ua = navigator.userAgent || '';
  const isWindows = /win/i.test(platform) || /Windows/i.test(ua);
  const isMac = /mac/i.test(platform) || /Mac OS X/i.test(ua);

  if (isWindows) {
    suggest('windows');
    return;
  }
  if (!isMac) return; // Linux 등 — 아무것도 강조하지 않는다

  suggest('mac-arm64');
  if (uaData && typeof uaData.getHighEntropyValues === 'function') {
    uaData
      .getHighEntropyValues(['architecture'])
      .then((info) => {
        if (info && info.architecture === 'x86') suggest('mac-intel');
      })
      .catch(() => {});
  }
})();
