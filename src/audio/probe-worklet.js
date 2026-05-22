class BandProbeProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.port.postMessage({
      type: "ready",
      hasTextDecoder: typeof TextDecoder !== "undefined"
    });
  }

  process() {
    return false;
  }
}

registerProcessor("band-probe-processor", BandProbeProcessor);
