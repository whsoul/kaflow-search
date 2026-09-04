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

/* 데모 토글 — 체크하면 위의 세 버튼이 그대로 데모 빌드를 가리킨다.
 *
 * 데모와 제품은 다른 물건이 아니라 클러스터를 들고 왔는지만 다르다. 버튼을 따로 만들지
 * 않고 같은 버튼의 목적지를 바꾸는 이유가 그것이다. 부수적으로 OS 감지가 빗나가도
 * 나머지 버튼이 그대로 보여서 막다른 길이 생기지 않는다 — 링크 하나로 두었을 때는
 * Windows·Linux 방문자와 Intel Mac 의 Safari 사용자가 arm64 dmg 를 받았다
 * (아키텍처 판별에 쓰는 navigator.userAgentData 가 Chromium 전용이라).
 *
 * 라벨까지 바꾸는 것이 중요하다. 체크 사실을 잊고 제품인 줄 알며 데모를 받는 것이
 * 이 UI 의 유일한 실패 방식이라, 버튼 자신이 무엇을 주는지 말해야 한다. */
(function () {
  const toggle = document.querySelector('[data-demo-toggle]');
  const row = document.querySelector('.dl-buttons');
  const buttons = Array.from(document.querySelectorAll('.dl-button'));
  if (!toggle || !row || !buttons.length) return;

  buttons.forEach((b) => {
    const small = b.querySelector('small');
    b.dataset.prodHref = b.getAttribute('href');
    b.dataset.prodSmall = small ? small.textContent : '';
  });

  const apply = () => {
    const demo = toggle.checked;
    buttons.forEach((b) => {
      const small = b.querySelector('small');
      b.setAttribute('href', demo ? `/download/demo-${b.dataset.platform}` : b.dataset.prodHref);
      if (small) small.textContent = demo ? `Demo · ${b.dataset.prodSmall}` : b.dataset.prodSmall;
    });
    row.classList.toggle('is-demo', demo);
  };

  toggle.addEventListener('change', apply);
  // 뒤로가기나 새로고침에서 브라우저가 체크 상태를 되살리므로, 적재 시점에도 맞춰준다.
  apply();
})();

/* 히어로 데모 영상 — 재생/일시정지 토글 + 동작 줄이기 설정 대응.
 *
 * autoplay 속성을 쓰지 않는다: 그 속성으로 시작해버리면 "동작 줄이기"를 켠 사용자에게
 * 이미 재생된 뒤에야 멈출 수 있어 한 번은 움직이는 화면을 보게 된다. 재생 여부를 여기서
 * 판단해 처음부터 시작하지 않는다.
 *
 * 마크업에는 controls 를 남겨둔다 — JS 가 없으면 네이티브 컨트롤이 폴백으로 남고,
 * JS 가 있으면 그것을 걷어내고 작은 토글 버튼으로 바꾼다. */
(function () {
  const video = document.querySelector('video.hero-quick-search');
  const stage = video && video.closest('.hero-shot-stage');
  if (!video || !stage) return;

  video.removeAttribute('controls');

  const ICON = {
    play: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5v14l11-7z"/></svg>',
    pause: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 5h4v14H7zM13 5h4v14h-4z"/></svg>',
  };
  const label = {
    play: video.dataset.playLabel || 'Play',
    pause: video.dataset.pauseLabel || 'Pause',
  };

  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'hero-video-toggle';
  stage.appendChild(button);

  function sync() {
    const playing = !video.paused && !video.ended;
    button.innerHTML = playing ? ICON.pause : ICON.play;
    button.setAttribute('aria-label', playing ? label.pause : label.play);
  }

  button.addEventListener('click', () => {
    if (video.paused) video.play().catch(() => {});
    else video.pause();
  });
  video.addEventListener('play', sync);
  video.addEventListener('pause', sync);

  // 동작 줄이기 설정이면 시작하지 않는다 — 포스터가 그대로 보이고, 원하면 버튼으로 재생한다.
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  if (!reduceMotion.matches) {
    // 자동재생이 막히면(모바일 등) 조용히 실패하고 포스터 + 재생 버튼 상태로 남는다.
    video.play().catch(() => {});
  }
  sync();
})();
