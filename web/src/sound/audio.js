// Bộ tổng hợp âm thanh Web Audio API 0-dependency
// Định danh đơn từ tiếng Anh: audio, context, place, capture, check, win, ctx, time, tone, gain, ring, amp, filter, size, buffer, data, step, noise, notes, pitch, index, start

let audio = null;

// Khởi tạo hoặc khôi phục ngữ cảnh Web Audio API
function context() {
  if (!audio) {
    const Ctx = window.AudioContext || window.webkitAudioContext;
    if (Ctx) {
      audio = new Ctx();
    }
  }
  if (audio && audio.state === 'suspended') {
    audio.resume();
  }
  return audio;
}

// Âm thanh gõ quân cờ xuống bàn cờ (tiếng gỗ trầm đĩnh đạc)
export function place() {
  const ctx = context();
  if (!ctx) return;

  const time = ctx.currentTime;

  const tone = ctx.createOscillator();
  const gain = ctx.createGain();
  tone.type = 'sine';
  tone.frequency.setValueAtTime(320, time);
  tone.frequency.exponentialRampToValueAtTime(80, time + 0.04);

  gain.gain.setValueAtTime(0.8, time);
  gain.gain.exponentialRampToValueAtTime(0.001, time + 0.04);

  const ring = ctx.createOscillator();
  const amp = ctx.createGain();
  ring.type = 'triangle';
  ring.frequency.setValueAtTime(140, time);
  ring.frequency.exponentialRampToValueAtTime(50, time + 0.03);

  amp.gain.setValueAtTime(0.4, time);
  amp.gain.exponentialRampToValueAtTime(0.001, time + 0.03);

  const filter = ctx.createBiquadFilter();
  filter.type = 'lowpass';
  filter.frequency.value = 1200;

  tone.connect(gain);
  ring.connect(amp);
  gain.connect(filter);
  amp.connect(filter);
  filter.connect(ctx.destination);

  tone.start(time);
  ring.start(time);
  tone.stop(time + 0.045);
  ring.stop(time + 0.045);
}

// Âm thanh ăn quân (va chạm đập quân giòn giã)
export function capture() {
  const ctx = context();
  if (!ctx) return;

  const time = ctx.currentTime;

  const tone = ctx.createOscillator();
  const gain = ctx.createGain();
  tone.type = 'sine';
  tone.frequency.setValueAtTime(480, time);
  tone.frequency.exponentialRampToValueAtTime(60, time + 0.06);

  gain.gain.setValueAtTime(1.0, time);
  gain.gain.exponentialRampToValueAtTime(0.001, time + 0.06);

  const size = ctx.sampleRate * 0.05;
  const buffer = ctx.createBuffer(1, size, ctx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let step = 0; step < size; step++) {
    data[step] = Math.random() * 2 - 1;
  }

  const noise = ctx.createBufferSource();
  noise.buffer = buffer;

  const amp = ctx.createGain();
  amp.gain.setValueAtTime(0.7, time);
  amp.gain.exponentialRampToValueAtTime(0.001, time + 0.05);

  const filter = ctx.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.value = 1800;
  filter.Q.value = 2.5;

  tone.connect(gain);
  gain.connect(ctx.destination);

  noise.connect(filter);
  filter.connect(amp);
  amp.connect(ctx.destination);

  tone.start(time);
  noise.start(time);
  tone.stop(time + 0.065);
  noise.stop(time + 0.055);
}

// Âm thanh cảnh báo Chiếu Tướng (âm ngân cao vút)
export function check() {
  const ctx = context();
  if (!ctx) return;

  const time = ctx.currentTime;

  const tone = ctx.createOscillator();
  const gain = ctx.createGain();
  tone.type = 'sine';
  tone.frequency.setValueAtTime(880, time);
  tone.frequency.exponentialRampToValueAtTime(440, time + 0.25);

  gain.gain.setValueAtTime(0.6, time);
  gain.gain.exponentialRampToValueAtTime(0.001, time + 0.25);

  tone.connect(gain);
  gain.connect(ctx.destination);

  tone.start(time);
  tone.stop(time + 0.26);
}

// Âm thanh khúc khải hoàn chiến thắng (Fanfare 4 nốt nhạc)
export function win() {
  const ctx = context();
  if (!ctx) return;

  const notes = [523.25, 659.25, 783.99, 1046.50];
  const time = ctx.currentTime;

  notes.forEach((pitch, index) => {
    const tone = ctx.createOscillator();
    const gain = ctx.createGain();
    tone.type = 'triangle';
    tone.frequency.value = pitch;

    const start = time + index * 0.1;
    gain.gain.setValueAtTime(0.5, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.3);

    tone.connect(gain);
    gain.connect(ctx.destination);

    tone.start(start);
    tone.stop(start + 0.31);
  });
}
