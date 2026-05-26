import wasmBinaryUrl from "../wasm/band_engine/band_engine_bg.wasm?url";
import wasmModuleUrl from "../wasm/band_engine/band_engine.js?url";
import bufferWorkletSource from "./buffer-worklet.js?raw";
import workletSource from "./band-worklet.js?raw";
import probeWorkletSource from "./probe-worklet.js?raw";
import {
  createFallbackLoopSource,
  renderInterleavedLoop
} from "./fallback-renderer.js";

const FADE_SECONDS = 0.035;
const MASTER_LEVEL = 1;
let workletObjectUrl = "";
let bufferWorkletObjectUrl = "";
let probeWorkletObjectUrl = "";

export class AudioController {
  constructor({ seed, onStateChange, onStatus, onError }) {
    this.seed = seed;
    this.onStateChange = onStateChange;
    this.onStatus = onStatus;
    this.onError = onError;
    this.context = null;
    this.master = null;
    this.workletNode = null;
    this.bufferSource = null;
    this.state = "stopped";
    this.path = "none";
    this.generation = 0;
    this.liveWorkletCapable = null;
  }

  async play() {
    if (this.state === "playing" || this.state === "starting") {
      return;
    }

    const generation = ++this.generation;
    this.setState("starting");
    this.onStatus?.("Starting audio.");
    this.onError?.("");

    try {
      await this.ensureContext();
      await this.context.resume();
      this.createMaster();

      try {
        await this.startWorklet(generation);
      } catch (workletError) {
        console.warn("AudioWorklet paths failed; using AudioBuffer fallback.", workletError);
        await this.cleanupWorklet();
        await this.startFallback(generation);
      }

      if (generation !== this.generation) {
        await this.stop();
        return;
      }

      this.fadeMaster(MASTER_LEVEL);
      this.setState("playing");
      this.onStatus?.("The generated loop is playing.");
    } catch (error) {
      await this.stop();
      this.setState("stopped");
      this.onError?.(friendlyAudioError(error));
      this.onStatus?.("Audio could not start.");
    }
  }

  async stop() {
    if (this.state === "stopped") {
      return;
    }

    this.generation += 1;
    this.setState("stopping");
    this.fadeMaster(0);
    await delay(FADE_SECONDS * 1000 + 20);

    if (this.workletNode) {
      this.workletNode.port.postMessage({ type: "stop" });
      this.workletNode.disconnect();
      this.workletNode = null;
    }

    if (this.bufferSource) {
      try {
        this.bufferSource.stop();
      } catch {
        // BufferSourceNode may already be stopped after rapid user interaction.
      }
      this.bufferSource.disconnect();
      this.bufferSource = null;
    }

    if (this.master) {
      this.master.disconnect();
      this.master = null;
    }

    this.path = "none";
    this.setState("stopped");
    this.onStatus?.("Audio is stopped and reset to beat one.");
  }

  async destroy() {
    await this.stop();
    if (this.context && this.context.state !== "closed") {
      await this.context.close();
    }
    this.context = null;
  }

  async ensureContext() {
    if (this.context && this.context.state !== "closed") {
      return;
    }

    const AudioContextClass = globalThis.AudioContext || globalThis.webkitAudioContext;
    if (!AudioContextClass) {
      throw new Error("This browser does not support AudioContext.");
    }
    this.context = new AudioContextClass({ latencyHint: "interactive" });
  }

  createMaster() {
    if (this.master) {
      this.master.disconnect();
    }
    this.master = this.context.createGain();
    this.master.gain.setValueAtTime(0, this.context.currentTime);
    this.master.connect(this.context.destination);
  }

  async startWorklet(generation) {
    if (!this.context.audioWorklet) {
      throw new Error("AudioWorklet is not supported.");
    }

    if (await this.canAttemptLiveWasmWorklet()) {
      try {
        await this.startLiveWasmWorklet(generation);
        return;
      } catch (error) {
        this.liveWorkletCapable = false;
        if (import.meta.env.DEV) {
          console.debug("Live WASM AudioWorklet unavailable; using pre-rendered worklet.", error);
        }
        await this.cleanupWorklet();
      }
    }

    await this.startBufferedWorklet(generation);
  }

  async canAttemptLiveWasmWorklet() {
    if (this.liveWorkletCapable !== null) {
      return this.liveWorkletCapable;
    }

    try {
      await this.context.audioWorklet.addModule(getProbeWorkletModuleUrl());
      const node = new AudioWorkletNode(this.context, "band-probe-processor", {
        numberOfInputs: 0,
        numberOfOutputs: 1,
        outputChannelCount: [1]
      });
      const result = await waitForWorkletReady(node);
      node.disconnect();
      this.liveWorkletCapable = result.hasTextDecoder === true;
    } catch {
      this.liveWorkletCapable = false;
    }

    return this.liveWorkletCapable;
  }

  async startLiveWasmWorklet(generation) {
    await this.context.audioWorklet.addModule(getWorkletModuleUrl());
    if (generation !== this.generation) {
      return;
    }

    const node = new AudioWorkletNode(this.context, "band-processor", {
      numberOfInputs: 0,
      numberOfOutputs: 1,
      outputChannelCount: [2],
      processorOptions: {
        seedHi: this.seed.hi,
        seedLo: this.seed.lo,
        wasmModuleUrl,
        wasmBinaryUrl
      }
    });

    const readyPromise = waitForWorkletReady(node);
    node.connect(this.master);
    this.workletNode = node;
    await readyPromise;

    if (generation !== this.generation) {
      node.disconnect();
      if (this.workletNode === node) {
        this.workletNode = null;
      }
      return;
    }

    node.port.postMessage({ type: "start" });
    this.path = "worklet";
  }

  async startBufferedWorklet(generation) {
    const interleaved = renderInterleavedLoop(this.context, this.seed);
    await this.context.audioWorklet.addModule(getBufferWorkletModuleUrl());
    if (generation !== this.generation) {
      return;
    }

    const node = new AudioWorkletNode(this.context, "band-buffer-processor", {
      numberOfInputs: 0,
      numberOfOutputs: 1,
      outputChannelCount: [2],
      processorOptions: {
        buffer: interleaved
      }
    });

    const readyPromise = waitForWorkletReady(node);
    node.connect(this.master);
    this.workletNode = node;
    await readyPromise;

    if (generation !== this.generation) {
      node.disconnect();
      if (this.workletNode === node) {
        this.workletNode = null;
      }
      return;
    }

    node.port.postMessage({ type: "start" });
    this.path = "worklet-buffer";
  }

  async startFallback(generation) {
    const source = await createFallbackLoopSource(this.context, this.seed);
    if (generation !== this.generation) {
      source.disconnect();
      return;
    }

    source.connect(this.master);
    source.start(0);
    this.bufferSource = source;
    this.path = "buffer";
    this.onStatus?.("The generated loop is playing with the compatibility renderer.");
  }

  async cleanupWorklet() {
    if (this.workletNode) {
      this.workletNode.port.postMessage({ type: "stop" });
      this.workletNode.disconnect();
      this.workletNode = null;
    }
  }

  fadeMaster(value) {
    if (!this.master) {
      return;
    }
    const now = this.context.currentTime;
    this.master.gain.cancelScheduledValues(now);
    this.master.gain.setValueAtTime(this.master.gain.value, now);
    this.master.gain.linearRampToValueAtTime(value, now + FADE_SECONDS);
  }

  setState(state) {
    this.state = state;
    this.onStateChange?.(state);
  }
}

function getWorkletModuleUrl() {
  if (!workletObjectUrl) {
    const absoluteWasmModuleUrl = new URL(wasmModuleUrl, globalThis.location.href).href;
    const absoluteWasmBinaryUrl = new URL(wasmBinaryUrl, globalThis.location.href).href;
    const moduleHeader = [
      `import initWasm, { Engine } from ${JSON.stringify(absoluteWasmModuleUrl)};`,
      `const WASM_BINARY_URL = ${JSON.stringify(absoluteWasmBinaryUrl)};`
    ].join("\n");
    workletObjectUrl = URL.createObjectURL(
      new Blob([moduleHeader, "\n", workletSource], { type: "text/javascript" })
    );
  }
  return workletObjectUrl;
}

function getBufferWorkletModuleUrl() {
  if (!bufferWorkletObjectUrl) {
    bufferWorkletObjectUrl = URL.createObjectURL(
      new Blob([bufferWorkletSource], { type: "text/javascript" })
    );
  }
  return bufferWorkletObjectUrl;
}

function getProbeWorkletModuleUrl() {
  if (!probeWorkletObjectUrl) {
    probeWorkletObjectUrl = URL.createObjectURL(
      new Blob([probeWorkletSource], { type: "text/javascript" })
    );
  }
  return probeWorkletObjectUrl;
}

function waitForWorkletReady(node) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error("AudioWorklet WASM initialization timed out."));
    }, 3500);

    const cleanup = () => {
      clearTimeout(timeout);
      node.port.removeEventListener("message", onMessage);
    };

    const onMessage = (event) => {
      const message = event.data || {};
      if (message.type === "ready") {
        cleanup();
        resolve(message);
      }
      if (message.type === "error") {
        cleanup();
        reject(new Error(message.message || "AudioWorklet initialization failed."));
      }
    };

    node.port.addEventListener("message", onMessage);
    node.port.start();
  });
}

function friendlyAudioError(error) {
  const message = error?.message || String(error);
  if (/AudioContext/i.test(message)) {
    return "Audio is not available in this browser.";
  }
  if (/autoplay|gesture|permission/i.test(message)) {
    return "Audio could not start. Press Play again after the page is active.";
  }
  return "Audio could not start. This browser may not support the required Web Audio features.";
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
