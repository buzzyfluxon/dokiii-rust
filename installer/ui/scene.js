(function () {
  var canvas = document.getElementById('stars');
  var ctx = canvas.getContext('2d');
  var dpr = Math.min(window.devicePixelRatio || 1, 2);

  function rng(seed) {
    var a = seed >>> 0;
    return function () {
      a = (a + 0x6d2b79f5) >>> 0;
      var t = a;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  }

  var rand = rng(20261126);
  var stars = [];
  for (var i = 0; i < 90; i++) {
    stars.push({
      x: rand(),
      y: rand() * 0.9,
      r: 0.5 + rand() * 0.9,
      phase: rand() * Math.PI * 2,
      speed: 0.5 + rand() * 1.4
    });
  }

  function resize() {
    canvas.width = Math.round(window.innerWidth * dpr);
    canvas.height = Math.round(window.innerHeight * dpr);
  }

  function draw(time) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (var k = 0; k < stars.length; k++) {
      var s = stars[k];
      var wave = 0.5 + 0.5 * Math.sin(time * s.speed + s.phase);
      var alpha = 0.12 + 0.6 * wave * (1 - s.y * 0.6);
      ctx.fillStyle = 'rgba(220, 228, 255, ' + alpha.toFixed(2) + ')';
      ctx.beginPath();
      ctx.arc(s.x * canvas.width, s.y * canvas.height, s.r * dpr, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  resize();
  var start = performance.now();
  draw(0);
  window.addEventListener('resize', function () {
    resize();
    draw((performance.now() - start) / 1000);
  });
  setInterval(function () {
    if (document.hidden) return;
    draw((performance.now() - start) / 1000);
  }, 250);

  var dock = document.getElementById('mini-dock');
  var icons = [
    'icons/terminal.png',
    'icons/brave.png',
    'logo.png',
    'icons/spotify.png',
    'icons/vscode.png',
    'icons/github.png'
  ];

  var BASE = 44;
  var GAP = 10;
  var PEAK = 0.62;
  var SPREAD = 58;
  var tiles = icons.map(function (icon) {
    var el = document.createElement('div');
    el.className = 'tile';
    el.style.width = BASE + 'px';
    el.style.height = BASE + 'px';
    var img = document.createElement('img');
    img.src = icon;
    img.draggable = false;
    el.appendChild(img);
    dock.appendChild(el);
    return el;
  });

  var hoverX = null;
  dock.addEventListener('mousemove', function (event) {
    var rect = dock.getBoundingClientRect();
    hoverX = event.clientX - rect.left;
  });
  dock.addEventListener('mouseleave', function () {
    hoverX = null;
  });

  function layout(time) {
    var width = dock.clientWidth;
    var restTotal = tiles.length * BASE + (tiles.length - 1) * GAP;
    var restStart = (width - restTotal) / 2;
    var focus = hoverX;
    if (focus === null) {
      var sweep = 0.5 + 0.5 * Math.sin(time * 0.9);
      focus = restStart + sweep * restTotal;
    }
    var scales = [];
    var total = 0;
    for (var i = 0; i < tiles.length; i++) {
      var center = restStart + i * (BASE + GAP) + BASE / 2;
      var d = (center - focus) / SPREAD;
      var scale = 1 + PEAK * Math.exp(-d * d);
      scales.push(scale);
      total += BASE * scale + (i < tiles.length - 1 ? GAP : 0);
    }
    var x = (width - total) / 2;
    for (var j = 0; j < tiles.length; j++) {
      var size = BASE * scales[j];
      tiles[j].style.transform = 'translate3d(' + x.toFixed(1) + 'px,0,0) scale(' + scales[j].toFixed(3) + ')';
      x += size + GAP;
    }
  }

  function frame(now) {
    if (!document.hidden) layout(now / 1000);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
})();
