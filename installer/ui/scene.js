(function () {
  var canvas = document.getElementById('scene');
  var ctx = canvas.getContext('2d');
  var W = canvas.width;
  var H = canvas.height;

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

  var rand = rng(20261125);

  var bayer = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5]
  ];

  var sky = [
    '#0c0720', '#110a2a', '#170d33', '#1e1140', '#28164f', '#37195d',
    '#4b2067', '#642772', '#7f2f7a', '#9c397f', '#b94883', '#d15c87',
    '#e47a8b', '#f2a094'
  ];

  var bg = document.createElement('canvas');
  bg.width = W;
  bg.height = H;
  var b = bg.getContext('2d');

  var horizon = 122;
  for (var y = 0; y < H; y++) {
    for (var x = 0; x < W; x++) {
      var t = Math.min(1, y / horizon) * (sky.length - 1);
      var d = bayer[y & 3][x & 3] / 16 - 0.5;
      var i = Math.max(0, Math.min(sky.length - 1, Math.floor(t + d * 0.95 + 0.5)));
      b.fillStyle = sky[i];
      b.fillRect(x, y, 1, 1);
    }
  }

  b.fillStyle = 'rgba(255, 196, 170, 0.55)';
  b.fillRect(0, 111, 96, 2);
  b.fillStyle = 'rgba(255, 214, 190, 0.35)';
  b.fillRect(0, 113, 70, 1);

  function ridge(base, amp, step, seed) {
    var r = rng(seed);
    var heights = new Array(W);
    var current = base - r() * amp;
    var target = current;
    for (var x = 0; x < W; x++) {
      if (x % step === 0) target = base - r() * amp;
      current += Math.sign(target - current) * Math.min(Math.abs(target - current), 1);
      heights[x] = Math.round(current);
    }
    return heights;
  }

  function fillRidge(heights, color) {
    b.fillStyle = color;
    for (var x = 0; x < W; x++) {
      b.fillRect(x, heights[x], 1, H - heights[x]);
    }
  }

  function pine(x, y, h, color) {
    b.fillStyle = color;
    b.fillRect(x, y - 2, 1, 2);
    for (var row = 0; row < h; row++) {
      var half = Math.floor((row + 1) / 2);
      b.fillRect(x - half, y - 2 - h + row + 1, half * 2 + 1, 1);
    }
  }

  function roundTree(x, y, r, color) {
    b.fillStyle = color;
    b.fillRect(x, y - 3, 1, 3);
    for (var dy = -r; dy <= r; dy++) {
      var w = Math.round(Math.sqrt(r * r - dy * dy));
      b.fillRect(x - w, y - 3 - r + dy, w * 2 + 1, 1);
    }
  }

  var far = ridge(120, 9, 7, 11);
  fillRidge(far, '#4d2b6e');

  var farTrees = rng(5);
  for (var f = 0; f < 34; f++) {
    var fx = Math.floor(farTrees() * W);
    pine(fx, far[fx] + 1, 3 + Math.floor(farTrees() * 2), '#4d2b6e');
  }

  var mid = ridge(130, 11, 9, 23);
  fillRidge(mid, '#341a52');

  var midTrees = rng(9);
  for (var m = 0; m < 24; m++) {
    var mx = Math.floor(midTrees() * W);
    if (midTrees() > 0.35) pine(mx, mid[mx] + 1, 5 + Math.floor(midTrees() * 3), '#341a52');
    else roundTree(mx, mid[mx] + 1, 3, '#341a52');
  }

  var near = ridge(141, 9, 11, 37);
  fillRidge(near, '#241239');

  var nearTrees = rng(13);
  for (var n = 0; n < 16; n++) {
    var nx = Math.floor(nearTrees() * W);
    if (nearTrees() > 0.4) pine(nx, near[nx] + 1, 8 + Math.floor(nearTrees() * 5), '#241239');
    else roundTree(nx, near[nx] + 1, 4, '#241239');
  }

  var front = ridge(152, 6, 13, 51);
  fillRidge(front, '#170b27');

  var frontTrees = rng(17);
  for (var q = 0; q < 8; q++) {
    var qx = Math.floor(frontTrees() * W);
    pine(qx, front[qx] + 1, 6 + Math.floor(frontTrees() * 4), '#170b27');
  }

  var stars = [];
  for (var s = 0; s < 96; s++) {
    stars.push({
      x: Math.floor(rand() * W),
      y: Math.floor(rand() * 92),
      big: rand() > 0.9,
      phase: rand() * Math.PI * 2,
      speed: 0.4 + rand() * 1.2
    });
  }

  function starAlpha(star, time) {
    var base = 0.35 + 0.65 * (1 - star.y / 110);
    var wave = 0.5 + 0.5 * Math.sin(time * star.speed + star.phase);
    return Math.max(0.12, base * (0.45 + 0.55 * wave));
  }

  function draw(time) {
    ctx.drawImage(bg, 0, 0);
    for (var k = 0; k < stars.length; k++) {
      var star = stars[k];
      var a = starAlpha(star, time);
      ctx.fillStyle = 'rgba(236, 228, 255, ' + a.toFixed(2) + ')';
      ctx.fillRect(star.x, star.y, 1, 1);
      if (star.big) {
        ctx.fillStyle = 'rgba(236, 228, 255, ' + (a * 0.55).toFixed(2) + ')';
        ctx.fillRect(star.x - 1, star.y, 1, 1);
        ctx.fillRect(star.x + 1, star.y, 1, 1);
        ctx.fillRect(star.x, star.y - 1, 1, 1);
        ctx.fillRect(star.x, star.y + 1, 1, 1);
      }
    }
  }

  var start = performance.now();
  draw(0);
  setInterval(function () {
    if (document.hidden) return;
    draw((performance.now() - start) / 1000);
  }, 250);
})();
