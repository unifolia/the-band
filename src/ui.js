export function createAppShell(root) {
  root.innerHTML = "";

  const app = document.createElement("main");
  app.className = "app-shell";
  app.setAttribute("aria-labelledby", "app-title");

  const title = document.createElement("h1");
  title.id = "app-title";
  title.className = "app-title";
  title.textContent = "The Band";

  const description = document.createElement("p");
  description.id = "band-description";
  description.className = "sr-only";

  const status = document.createElement("p");
  status.id = "playback-status";
  status.className = "sr-only";
  status.setAttribute("aria-live", "polite");
  status.textContent = "The generated band is ready. Audio is stopped.";

  const stage = document.createElement("section");
  stage.className = "band-stage";
  stage.setAttribute("aria-label", "Generated swamp band members");

  const stageCopy = document.createElement("div");
  stageCopy.className = "stage-copy";

  const stageText = document.createElement("p");
  stageText.className = "stage-text";
  stageText.textContent =
    "The\u00a0Mucks are the most popular band in Lizard\u00a0Marsh. Whether they're playing through their vast catalog of originals or classic covers, The\u00a0Mucks are sure to provide a slime of a time.";

  const stageTagline = document.createElement("p");
  stageTagline.className = "stage-tagline";
  stageTagline.textContent = `"The\u00a0Mucks! Crawling out of the bog with swampy riffs and squelchy grooves!"`;

  stageCopy.append(stageText, stageTagline);
  stage.append(stageCopy);

  const spriteSlots = Array.from({ length: 4 }, (_, index) => {
    const slot = document.createElement("div");
    slot.className = "band-member";
    slot.style.setProperty("--entry-delay", `${140 + index * 115}ms`);
    slot.setAttribute("aria-hidden", "true");
    stage.append(slot);
    return slot;
  });

  const playButton = document.createElement("button");
  playButton.type = "button";
  playButton.className = "play-button";
  playButton.textContent = "Play";
  playButton.setAttribute("aria-pressed", "false");
  playButton.setAttribute(
    "aria-describedby",
    "band-description playback-status",
  );

  const error = document.createElement("p");
  error.className = "app-error";
  error.hidden = true;
  error.setAttribute("role", "alert");

  const footer = document.createElement("footer");
  const footerLink = document.createElement("a");
  footerLink.href = "https://midi.engineering/";
  footerLink.textContent = "\uD801\uDE66 MIDI Engineering";
  footer.append(footerLink);

  app.append(title, description, stage, playButton, status, error, footer);
  root.append(app);

  return {
    playButton,
    setSprites(sprites, names = []) {
      sprites.forEach((sprite, index) => {
        const nameTag = document.createElement("span");
        nameTag.className = "member-name";
        nameTag.textContent = names[index] || "";
        spriteSlots[index].replaceChildren(nameTag, sprite);
      });
    },
    setDescription(text) {
      description.textContent = text;
    },
    setStatus(text) {
      status.textContent = text;
    },
    setBusy(isBusy) {
      playButton.disabled = isBusy;
      playButton.setAttribute("aria-busy", String(isBusy));
    },
    setPlaying(isPlaying) {
      playButton.textContent = isPlaying ? "Stop" : "Play";
      playButton.setAttribute("aria-pressed", String(isPlaying));
      playButton.classList.toggle("is-playing", isPlaying);
    },
    setError(message) {
      if (!message) {
        error.hidden = true;
        error.textContent = "";
        return;
      }
      error.hidden = false;
      error.textContent = message;
    },
  };
}
