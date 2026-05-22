class BandProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.engine = null;
    this.output = null;
    this.ready = false;
    this.playing = false;
    this.failed = false;

    this.port.onmessage = (event) => {
      const message = event.data || {};
      if (message.type === "start" && this.ready) {
        this.engine.reset();
        this.playing = true;
      }
      if (message.type === "stop") {
        this.playing = false;
        if (this.ready) {
          this.engine.reset();
        }
      }
    };

    this.initialize(options.processorOptions || {}).catch((error) => {
      this.failed = true;
      this.port.postMessage({
        type: "error",
        message: error?.message || "AudioWorklet WASM initialization failed."
      });
    });
  }

  async initialize(options) {
    await initWasm(WASM_BINARY_URL);
    const engine = new Engine(options.seedHi >>> 0, options.seedLo >>> 0, sampleRate);
    this.engine = engine;
    this.refreshOutputView();
    this.ready = true;
    this.port.postMessage({ type: "ready", loopSamples: engine.loop_samples() });
  }

  refreshOutputView() {
    if (!this.output || this.output.length !== this.engine.output_len()) {
      this.output = this.engine.output_view();
    }
  }

  process(_inputs, outputs) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1] || output[0];

    if (this.failed) {
      return false;
    }

    if (!this.ready || !this.playing) {
      left.fill(0);
      if (right !== left) {
        right.fill(0);
      }
      return true;
    }

    const frames = left.length;
    try {
      this.engine.render(frames);
      this.refreshOutputView();
      for (let frame = 0; frame < frames; frame += 1) {
        const offset = frame * 2;
        left[frame] = this.output[offset] || 0;
        right[frame] = this.output[offset + 1] || 0;
      }
    } catch (error) {
      this.failed = true;
      this.port.postMessage({
        type: "error",
        message: error?.message || "AudioWorklet render failed."
      });
      return false;
    }

    return true;
  }
}

registerProcessor("band-processor", BandProcessor);
