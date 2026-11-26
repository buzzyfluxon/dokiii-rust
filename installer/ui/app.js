(function () {
  var tauri = window.__TAURI__;
  var invoke = tauri.core.invoke;
  var listen = tauri.event.listen;
  var appWindow = tauri.window.getCurrentWindow();

  var panel = document.getElementById('panel');
  var eyebrow = document.getElementById('eyebrow');
  var title = document.getElementById('title');
  var lead = document.getElementById('lead');
  var pathEl = document.getElementById('path');
  var changeBtn = document.getElementById('btn-change');
  var shortcut = document.getElementById('shortcut');
  var autoLaunch = document.getElementById('autolaunch');
  var installBtn = document.getElementById('btn-install');
  var launchBtn = document.getElementById('btn-launch');
  var errorText = document.getElementById('error-text');
  var stepEls = Array.prototype.slice.call(document.querySelectorAll('.step'));
  var stepNames = ['Preparing', 'Installing', 'Finishing up'];

  var state = { path: '', existing: false, exe: '' };

  function setState(next) {
    panel.setAttribute('data-state', next);
  }

  function setPath(value) {
    state.path = value;
    pathEl.textContent = value;
    pathEl.title = value;
  }

  function setStep(index) {
    stepEls.forEach(function (el, i) {
      el.className = 'step' + (i < index ? ' done' : i === index ? ' active' : '');
    });
  }

  function showWelcome() {
    eyebrow.textContent = 'Setup';
    title.textContent = state.existing ? 'Update DOKIII' : 'Install DOKIII';
    lead.textContent = state.existing
      ? 'DOKIII is already on this account. It will be updated in place.'
      : 'Pick a folder and a couple of options, then you are done.';
    installBtn.textContent = state.existing ? 'Update' : 'Install';
    changeBtn.disabled = state.existing;
    setState('welcome');
  }

  function showProgress() {
    eyebrow.textContent = state.existing ? 'Updating' : 'Installing';
    title.textContent = 'Setting things up';
    lead.textContent = 'This only takes a moment.';
    setStep(0);
    setState('progress');
  }

  function showDone(started) {
    eyebrow.textContent = 'Finished';
    title.textContent = 'DOKIII is ready';
    lead.textContent = started
      ? 'DOKIII is starting. Look for the dock along your screen edge.'
      : 'Look for the dock along your screen edge once it starts.';
    launchBtn.style.display = started ? 'none' : '';
    setState('done');
  }

  function showError(message) {
    eyebrow.textContent = 'Problem';
    title.textContent = 'Setup did not finish';
    lead.textContent = 'Nothing is left half-installed that you need to remove.';
    errorText.textContent = message;
    setState('error');
  }

  function closeInstaller() {
    invoke('close_installer');
  }

  document.getElementById('btn-min').addEventListener('click', function () {
    appWindow.minimize();
  });
  document.getElementById('btn-close').addEventListener('click', closeInstaller);
  document.getElementById('btn-finish').addEventListener('click', closeInstaller);
  document.getElementById('btn-error-close').addEventListener('click', closeInstaller);

  changeBtn.addEventListener('click', function () {
    invoke('pick_folder', { current: state.path }).then(function (picked) {
      if (picked) setPath(picked);
    });
  });

  function runInstall() {
    showProgress();
    invoke('install', { path: state.path, desktopShortcut: shortcut.checked })
      .then(function (exe) {
        state.exe = exe;
        if (autoLaunch.checked) {
          return invoke('launch_app', { exe: exe }).then(
            function () {
              showDone(true);
            },
            function () {
              showDone(false);
            }
          );
        }
        showDone(false);
      })
      .catch(function (error) {
        showError(String(error));
      });
  }

  installBtn.addEventListener('click', runInstall);
  document.getElementById('btn-retry').addEventListener('click', runInstall);

  launchBtn.addEventListener('click', function () {
    invoke('launch_app', { exe: state.exe }).finally(closeInstaller);
  });

  listen('install:phase', function (event) {
    var index = stepNames.indexOf(event.payload);
    if (index >= 0) setStep(index);
  });

  invoke('get_defaults').then(function (defaults) {
    state.existing = defaults.existing;
    setPath(defaults.path);
    document.getElementById('version').textContent = 'Version ' + defaults.version;
    showWelcome();
  });
})();
