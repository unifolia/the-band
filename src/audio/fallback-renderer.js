import { render_loop as renderLoop } from "../wasm/band_engine/band_engine.js";

export function renderInterleavedLoop(audioContext, seed) {
  const interleaved = renderLoop(seed.hi, seed.lo, audioContext.sampleRate);
  if (!(interleaved instanceof Float32Array) || interleaved.length < 2) {
    throw new Error("Rust loop renderer returned no audio.");
  }
  if (interleaved.length % 2 !== 0) {
    throw new Error("Rust loop renderer returned malformed stereo audio.");
  }
  return interleaved;
}

export async function createFallbackLoopSource(audioContext, seed) {
  const interleaved = renderInterleavedLoop(audioContext, seed);
  const frameCount = interleaved.length / 2;
  const buffer = audioContext.createBuffer(2, frameCount, audioContext.sampleRate);
  const left = buffer.getChannelData(0);
  const right = buffer.getChannelData(1);

  for (let frame = 0; frame < frameCount; frame += 1) {
    const offset = frame * 2;
    left[frame] = interleaved[offset];
    right[frame] = interleaved[offset + 1];
  }

  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.loop = true;
  return source;
}
