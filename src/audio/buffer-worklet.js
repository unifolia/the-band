class BandBufferProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.buffer = options.processorOptions?.buffer || new Float32Array();
    this.frameCount = Math.floor(this.buffer.length / 2);
    this.position = 0;
    this.playing = false;

    this.port.onmessage = (event) => {
      const message = event.data || {};
      if (message.type === "start") {
        this.position = 0;
        this.playing = true;
      }
      if (message.type === "stop") {
        this.playing = false;
        this.position = 0;
      }
    };

    this.port.postMessage({
      type: this.frameCount > 0 ? "ready" : "error",
      message: this.frameCount > 0 ? "" : "Rendered loop buffer was empty."
    });
  }

  process(_inputs, outputs) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1] || output[0];

    if (!this.playing || this.frameCount === 0) {
      left.fill(0);
      if (right !== left) {
        right.fill(0);
      }
      return true;
    }

    for (let frame = 0; frame < left.length; frame += 1) {
      const offset = this.position * 2;
      left[frame] = this.buffer[offset] || 0;
      right[frame] = this.buffer[offset + 1] || 0;
      this.position += 1;
      if (this.position >= this.frameCount) {
        this.position = 0;
      }
    }

    return true;
  }
}

registerProcessor("band-buffer-processor", BandBufferProcessor);
