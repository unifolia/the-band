import initWasm, {
  generate_band_json as generateBandJson,
} from "./wasm/band_engine/band_engine.js";
import { AudioController } from "./audio/audio-controller.js";
import { generateSpriteNames } from "./names.js";
import { createSeed, formatSeed } from "./rng.js";
import { describeSprites, renderSprites } from "./sprites.js";
import { createAppShell } from "./ui.js";
import "./styles.css";

const root = document.querySelector("#app");
const ui = createAppShell(root);
let audioController = null;
let currentSeed = null;

bootstrap().catch((error) => {
  console.error("The Band failed to initialize.", error);
  ui.setBusy(false);
  ui.setPlaying(false);
  ui.setStatus("The generated band could not be created.");
  ui.setError("The band could not be generated in this browser.");
});

async function bootstrap() {
  await initWasm();
  await regenerateBand({ status: "Generating the band." });

  ui.playButton.addEventListener("click", async () => {
    if (!audioController) {
      return;
    }

    try {
      if (audioController.state === "playing") {
        await regenerateBand({
          closeExistingAudio: true,
          status: "Regenerating the band.",
        });
        return;
      }
      await audioController.play();
    } catch (error) {
      console.error("The Band interaction failed.", error);
      ui.setBusy(false);
      ui.setPlaying(false);
      ui.setStatus("The band could not be regenerated.");
      ui.setError("The band could not be regenerated in this browser.");
    }
  });

  window.addEventListener("pagehide", () => {
    audioController?.destroy();
  });
}

async function regenerateBand({ closeExistingAudio = false, status } = {}) {
  ui.setBusy(true);
  ui.setPlaying(false);
  ui.setError("");
  ui.setStatus(status || "Generating the band.");

  if (closeExistingAudio && audioController) {
    const previousController = audioController;
    audioController = null;
    await previousController.destroy();
    ui.setBusy(true);
    ui.setPlaying(false);
    ui.setStatus(status || "Generating the band.");
  }

  const seed = createSeed();
  const band = JSON.parse(generateBandJson(seed.hi, seed.lo));
  const names = generateSpriteNames(seed);
  currentSeed = seed;
  // console.info("The Band seed:", formatSeed(currentSeed));
  if (import.meta.env.DEV) {
    console.debug("The Band generated configuration:", band);
  }

  ui.setSprites(renderSprites(currentSeed, band), names);
  ui.setDescription(createAccessibleBandDescription(band, currentSeed, names));
  audioController = createAudioController(currentSeed);
  ui.setStatus("A new generated band is ready. Audio is stopped.");
  ui.setBusy(false);
}

function createAudioController(seed) {
  return new AudioController({
    seed,
    onStateChange: (state) => {
      const isBusy = state === "starting" || state === "stopping";
      ui.setBusy(isBusy);
      ui.setPlaying(state === "playing");
    },
    onStatus: ui.setStatus,
    onError: ui.setError,
  });
}

function createAccessibleBandDescription(band, seed, names) {
  const tonality = `${band.root.name} ${band.mode}`;
  const creatureSummary = describeSprites(seed, band, names);
  return `A generated four-member swamp fantasy band is ready: one percussionist and three instrumentalists. The music uses ${tonality}, a fixed tempo, a four-bar loop, synthesized percussion, and three generated instrumental phrases. The visible members are ${creatureSummary}. Press Play to start audio; press Stop to generate a new band.`;
}
