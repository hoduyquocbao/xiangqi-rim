import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { place, capture, check, win } from '../audio.js';

describe('Sound Audio Synthesizer Empirical Tests', () => {
  let mockAudioContext;
  let createdOscillators = [];
  let createdGains = [];
  let createdFilters = [];
  let createdBuffers = [];
  let createdBufferSources = [];

  beforeEach(() => {
    createdOscillators = [];
    createdGains = [];
    createdFilters = [];
    createdBuffers = [];
    createdBufferSources = [];

    if (!mockAudioContext) {
      mockAudioContext = {
        state: 'running',
        currentTime: 10,
        sampleRate: 44100,
        destination: {},
        resume: vi.fn().mockImplementation(() => {
          mockAudioContext.state = 'running';
          return Promise.resolve();
        }),
        createOscillator: vi.fn().mockImplementation(() => {
          const osc = {
            type: '',
            frequency: {
              value: 0,
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
          createdOscillators.push(osc);
          return osc;
        }),
        createGain: vi.fn().mockImplementation(() => {
          const gain = {
            gain: {
              value: 0,
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
          createdGains.push(gain);
          return gain;
        }),
        createBiquadFilter: vi.fn().mockImplementation(() => {
          const filter = {
            type: '',
            frequency: { value: 0 },
            Q: { value: 0 },
            connect: vi.fn(),
          };
          createdFilters.push(filter);
          return filter;
        }),
        createBuffer: vi.fn().mockImplementation((channels, length, sampleRate) => {
          const buf = {
            numberOfChannels: channels,
            length: length,
            sampleRate: sampleRate,
            data: new Float32Array(length),
            getChannelData: function (c) {
              return this.data;
            },
          };
          createdBuffers.push(buf);
          return buf;
        }),
        createBufferSource: vi.fn().mockImplementation(() => {
          const src = {
            buffer: null,
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
          createdBufferSources.push(src);
          return src;
        }),
      };
    }

    function MockAudioContextConstructor() {
      return mockAudioContext;
    }

    window.AudioContext = MockAudioContextConstructor;
  });

  afterEach(() => {
    delete window.AudioContext;
    delete window.webkitAudioContext;
  });

  it('place() creates sine and triangle oscillators with lowpass filter and ramps frequency/gain', () => {
    place();
    expect(createdOscillators.length).toBeGreaterThanOrEqual(2);
    const osc1 = createdOscillators[createdOscillators.length - 2];
    const osc2 = createdOscillators[createdOscillators.length - 1];
    expect(osc1.type).toBe('sine');
    expect(osc2.type).toBe('triangle');

    expect(osc1.frequency.setValueAtTime).toHaveBeenCalledWith(320, 10);
    expect(osc1.frequency.exponentialRampToValueAtTime).toHaveBeenCalledWith(80, 10.04);

    const gain1 = createdGains[createdGains.length - 2];
    expect(gain1.gain.setValueAtTime).toHaveBeenCalledWith(0.8, 10);
    expect(gain1.gain.exponentialRampToValueAtTime).toHaveBeenCalledWith(0.001, 10.04);

    const filter1 = createdFilters[createdFilters.length - 1];
    expect(filter1.type).toBe('lowpass');
    expect(filter1.frequency.value).toBe(1200);

    expect(osc1.start).toHaveBeenCalledWith(10);
    expect(osc1.stop).toHaveBeenCalledWith(10.045);
    expect(osc2.start).toHaveBeenCalledWith(10);
    expect(osc2.stop).toHaveBeenCalledWith(10.045);
  });

  it('capture() creates sine oscillator and white noise buffer source with bandpass filter', () => {
    capture();
    const osc = createdOscillators[createdOscillators.length - 1];
    expect(osc.type).toBe('sine');
    expect(osc.frequency.setValueAtTime).toHaveBeenCalledWith(480, 10);
    expect(osc.frequency.exponentialRampToValueAtTime).toHaveBeenCalledWith(60, 10.06);

    const buf = createdBuffers[createdBuffers.length - 1];
    expect(buf.length).toBe(44100 * 0.05);

    const src = createdBufferSources[createdBufferSources.length - 1];
    expect(src.buffer).toBe(buf);

    const filter = createdFilters[createdFilters.length - 1];
    expect(filter.type).toBe('bandpass');
    expect(filter.frequency.value).toBe(1800);
    expect(filter.Q.value).toBe(2.5);

    expect(osc.start).toHaveBeenCalledWith(10);
    expect(osc.stop).toHaveBeenCalledWith(10.065);
    expect(src.start).toHaveBeenCalledWith(10);
    expect(src.stop).toHaveBeenCalledWith(10.055);
  });

  it('check() creates high pitch sine oscillator starting at 880Hz', () => {
    check();
    const osc = createdOscillators[createdOscillators.length - 1];
    expect(osc.type).toBe('sine');
    expect(osc.frequency.setValueAtTime).toHaveBeenCalledWith(880, 10);
    expect(osc.frequency.exponentialRampToValueAtTime).toHaveBeenCalledWith(440, 10.25);
    expect(osc.start).toHaveBeenCalledWith(10);
    expect(osc.stop).toHaveBeenCalledWith(10.26);
  });

  it('win() plays 4 fanfare triangle pitch notes', () => {
    win();
    const oscs = createdOscillators.slice(-4);
    expect(oscs.length).toBe(4);
    const expectedPitches = [523.25, 659.25, 783.99, 1046.50];
    oscs.forEach((osc, idx) => {
      expect(osc.type).toBe('triangle');
      expect(osc.frequency.value).toBe(expectedPitches[idx]);
      expect(osc.start).toHaveBeenCalledWith(10 + idx * 0.1);
    });
  });

  it('resumes audio context if state is suspended', () => {
    mockAudioContext.state = 'suspended';
    place();
    expect(mockAudioContext.resume).toHaveBeenCalled();
  });
});
