(function () {
  var tauri = window.__TAURI__;
  var invoke = tauri.core.invoke;
  var listen = tauri.event.listen;
  var appWindow = tauri.window.getCurrentWindow();

  var card = document.getElementById('card');
  var title = document.getElementById('title');
  var lead = document.getElementById('lead');
  var pathEl = document.getElementById('path');
  var changeBtn = document.getElementById('btn-change');
  var shortcut = document.getElementById('shortcut');
  var installBtn = document.getElementById('btn-install');
  var phaseEl = document.getElementById('phase');
  var errorText = document.getElementById('error-text');

  var state = { path: '', existing: false, exe: '' };

  function setState(next) {
    card.setAttribute('data-state', next);
  }

  function setPath(value) {
    state.path = value;
    pathEl.textContent = value;
    pathEl.title = value;
  }

  function setVersion(version) {
    ['version', 'version-done', 'version-error'].forEach(function (id) {
      document.getElementById(id).textContent = 'Version ' + version;
    });
  }

  function showWelcome() {
    title.textContent = 'DOKIII';
    lead.textContent = state.existing
      ? 'DOKIII is already on this account. This will update it in place.'
      : 'Installs DOKIII for this account. No administrator needed.';
    installBtn.textContent = state.existing ? 'Update' : 'Install';
    changeBtn.disabled = state.existing;
    setState('welcome');
  }

  function showProgress() {
    title.textContent = state.existing ? 'Updating' : 'Installing';
    lead.textContent = 'This only takes a moment.';
    phaseEl.textContent = 'Preparing';
    setState('progress');
  }

  function showDone() {
    title.textContent = 'All set';
    lead.textContent = 'DOKIII is installed. Look for the dock along your screen edge.';
    setState('done');
  }

  function showError(message) {
    title.textContent = 'Setup failed';
    lead.textContent = 'Nothing was changed that you need to undo.';
    errorText.textContent = message;
    setState('error');
  }

  document.getElementById('btn-min').addEventListener('click', function () {
    appWindow.minimize();
  });
  document.getElementById('btn-close').addEventListener('click', function () {
    invoke('close_installer');
  });
  document.getElementById('btn-finish').addEventListener('click', function () {
    invoke('close_installer');
  });
  document.getElementById('btn-error-close').addEventListener('click', function () {
    invoke('close_installer');
  });

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
        showDone();
      })
      .catch(function (error) {
        showError(String(error));
      });
  }

  installBtn.addEventListener('click', runInstall);
  document.getElementById('btn-retry').addEventListener('click', runInstall);

  document.getElementById('btn-launch').addEventListener('click', function () {
    invoke('launch_app', { exe: state.exe }).finally(function () {
      invoke('close_installer');
    });
  });

  listen('install:phase', function (event) {
    phaseEl.textContent = event.payload;
  });

  invoke('get_defaults').then(function (defaults) {
    state.existing = defaults.existing;
    setPath(defaults.path);
    setVersion(defaults.version);
    showWelcome();
  });
})();
