import { deriveSeed, Rng } from "./rng.js";

const SVG_NS = "http://www.w3.org/2000/svg";

const PALETTES = [
  {
    skin: "#355f45",
    skinDark: "#203a2d",
    skinLight: "#6f8f4d",
    cloak: "#293924",
    accent: "#d1a447",
    eye: "#d6ff5f",
    mud: "#3a2a1c",
  },
  {
    skin: "#2b5d62",
    skinDark: "#16383a",
    skinLight: "#6ca59a",
    cloak: "#203126",
    accent: "#b9893a",
    eye: "#a9f06a",
    mud: "#412916",
  },
  {
    skin: "#6a5f31",
    skinDark: "#38311e",
    skinLight: "#9b924b",
    cloak: "#27361d",
    accent: "#c5bb70",
    eye: "#f0f279",
    mud: "#2d2119",
  },
  {
    skin: "#4a3b5a",
    skinDark: "#241e31",
    skinLight: "#817056",
    cloak: "#1c342d",
    accent: "#86a94b",
    eye: "#d8ff79",
    mud: "#3d2519",
  },
];

const TRAITS = [
  "frog oracle",
  "reed-haired bog keeper",
  "mushroom-capped marsh singer",
  "mud-armored tunnel pilgrim",
  "branch-antlered fen watcher",
  "slug-tailed lowland wanderer",
];

export function renderSprites(seed, band) {
  return Array.from({ length: 4 }, (_, index) => {
    const memberSeed = deriveSeed(seed, 0x5100n + BigInt(index * 97 + 11));
    const role = index === 0 ? "percussionist" : "instrumentalist";
    return createCreatureSprite(
      memberSeed.value,
      index,
      role,
      getMemberInstrument(band, index),
    );
  });
}

export function describeSprites(seed, band, names = []) {
  return Array.from({ length: 4 }, (_, index) => {
    const memberSeed = deriveSeed(seed, 0x5100n + BigInt(index * 97 + 11));
    const { trait } = prepareCreature(memberSeed.value, index);
    const instrument = formatInstrumentName(getMemberInstrument(band, index));
    const name = names[index] || `member ${index + 1}`;
    return `${name}, a ${trait} with ${instrument}`;
  }).join("; ");
}

function createCreatureSprite(seedValue, index, role, instrument) {
  const { rng, palette, trait } = prepareCreature(seedValue, index);
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", "0 0 32 32");
  svg.setAttribute("width", "32");
  svg.setAttribute("height", "32");
  svg.setAttribute("role", "img");
  svg.setAttribute(
    "aria-label",
    `${role} sprite: ${trait} with ${formatInstrumentName(instrument)}`,
  );
  svg.setAttribute("shape-rendering", "crispEdges");
  svg.classList.add("creature-sprite");

  addRect(svg, 0, 0, 32, 32, "transparent");
  addShadow(svg, palette);

  const bodyTop = rng.int(10, 13);
  const bodyBottom = rng.int(24, 27);
  const halfRows = buildBodyRows(rng, bodyTop, bodyBottom, index);
  for (const row of halfRows) {
    paintSymmetricRow(svg, row.y, row.left, row.right, palette.skin);
    if (rng.chance(0.42)) {
      addRect(svg, row.left + 1, row.y, 1, 1, palette.skinLight);
    }
    if (rng.chance(0.52)) {
      addRect(svg, 31 - row.left - 1, row.y, 1, 1, palette.skinDark);
    }
  }

  addHead(svg, rng, palette, trait, role);
  addEyes(svg, rng, palette, role);
  addArms(svg, rng, palette);
  addLegs(svg, rng, palette, bodyBottom);
  addAccessories(svg, rng, palette, trait, role);
  addInstrumentSprite(svg, instrument);

  return svg;
}

function getMemberInstrument(band, index) {
  if (index === 0) {
    return band?.percussion?.bank || "drums";
  }
  return band?.instruments?.[index - 1]?.bank || "Synth";
}

function formatInstrumentName(instrument) {
  return String(instrument)
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .toLowerCase();
}

function prepareCreature(seedValue, index) {
  const rng = new Rng(seedValue);
  const palette =
    PALETTES[(rng.int(0, PALETTES.length - 1) + index) % PALETTES.length];
  const trait = TRAITS[(rng.int(0, TRAITS.length - 1) + index) % TRAITS.length];
  return { rng, palette, trait };
}

function buildBodyRows(rng, top, bottom, index) {
  const rows = [];
  const belly = rng.int(4, 6);
  for (let y = top; y <= bottom; y += 1) {
    const progress = (y - top) / Math.max(1, bottom - top);
    const waist = Math.round(Math.sin(progress * Math.PI) * belly);
    const irregular = rng.chance(0.18) ? 1 : 0;
    const width = Math.max(
      3,
      3 + waist + irregular + (index === 0 && y % 3 === 0 ? 1 : 0),
    );
    rows.push({ y, left: 16 - width, right: 16 + width - 1 });
  }
  return rows;
}

function addHead(svg, rng, palette, trait, role) {
  const headWidth = trait.includes("mushroom") ? rng.int(8, 10) : rng.int(5, 8);
  const headTop = trait.includes("mushroom") ? 5 : rng.int(6, 8);
  const headBottom = rng.int(12, 15);
  for (let y = headTop; y <= headBottom; y += 1) {
    const taper = y === headTop || y === headBottom ? 1 : 0;
    const width = headWidth - taper;
    paintSymmetricRow(svg, y, 16 - width, 16 + width - 1, palette.skin);
  }

  if (trait.includes("frog")) {
    addRect(svg, 8, 7, 4, 4, palette.skin);
    addRect(svg, 20, 7, 4, 4, palette.skin);
  }

  if (role === "percussionist") {
    addRect(svg, 12, 4, 8, 2, palette.accent);
    addRect(svg, 11, 5, 10, 1, palette.skinDark);
  }
}

function addEyes(svg, rng, palette, role) {
  const eyeY = rng.int(9, 11);
  const eyeGap = rng.int(3, 4);
  addRect(svg, 16 - eyeGap, eyeY, 2, 2, palette.eye, "sprite-eye");
  addRect(svg, 16 + eyeGap - 1, eyeY, 2, 2, palette.eye, "sprite-eye");
  addRect(svg, 16 - eyeGap, eyeY + 1, 1, 1, "#08110d");
  addRect(svg, 16 + eyeGap, eyeY + 1, 1, 1, "#08110d");
  if (role === "percussionist") {
    addRect(svg, 15, eyeY + 3, 2, 1, palette.accent);
  } else if (rng.chance(0.5)) {
    addRect(svg, 14, eyeY + 4, 4, 1, palette.skinDark);
  }
}

function addArms(svg, rng, palette) {
  const y = rng.int(15, 18);
  const armColor = rng.chance(0.34) ? palette.cloak : palette.skinDark;
  addRect(svg, 5, y, 5, 2, armColor);
  addRect(svg, 22, y + rng.int(-1, 1), 5, 2, armColor);
  addRect(svg, 4, y + 2, 3, 2, palette.skin);
  addRect(svg, 26, y + 1, 3, 2, palette.skin);
}

function addLegs(svg, rng, palette, bodyBottom) {
  const footY = Math.min(30, bodyBottom + 1);
  addRect(svg, 10, bodyBottom, 4, 3, palette.skinDark);
  addRect(svg, 18, bodyBottom, 4, 3, palette.skinDark);
  addRect(svg, 8, footY + 1, 6, 2, palette.mud);
  addRect(svg, 18, footY + 1, 6, 2, palette.mud);
  if (rng.chance(0.55)) {
    addRect(svg, 23, bodyBottom - 1, 4, 2, palette.skinDark);
    addRect(svg, 26, bodyBottom, 3, 1, palette.skin);
  }
}

function addAccessories(svg, rng, palette, trait, role) {
  if (trait.includes("mushroom")) {
    addRect(svg, 9, 4, 14, 2, "#8f4f37");
    addRect(svg, 7, 6, 18, 3, "#713a32");
    addRect(svg, 11, 5, 2, 1, palette.accent);
    addRect(svg, 20, 7, 2, 1, palette.accent);
  }

  if (trait.includes("reed")) {
    for (let x = 11; x <= 21; x += 2) {
      addRect(
        svg,
        x,
        rng.int(2, 5),
        1,
        rng.int(5, 9),
        rng.chance(0.5) ? "#6e7b31" : "#9a8d3b",
      );
    }
  }

  if (trait.includes("branch")) {
    addRect(svg, 9, 5, 2, 5, "#5d4526");
    addRect(svg, 21, 5, 2, 5, "#5d4526");
    addRect(svg, 7, 5, 4, 1, "#5d4526");
    addRect(svg, 21, 4, 5, 1, "#5d4526");
  }

  if (trait.includes("mud-armored") || role === "percussionist") {
    addRect(svg, 11, 17, 10, 2, palette.mud);
    addRect(svg, 10, 20, 12, 2, palette.mud);
    addRect(svg, 13, 18, 2, 1, palette.accent);
  }

  if (trait.includes("slug")) {
    addRect(svg, 20, 25, 7, 2, palette.skinDark);
    addRect(svg, 25, 24, 3, 1, palette.skinLight);
  }

  if (rng.chance(0.62)) {
    const side = rng.chance(0.5) ? 6 : 24;
    addRect(svg, side, 13, 2, 6, palette.cloak);
    addRect(svg, side + (side < 16 ? -1 : 1), 18, 2, 4, palette.cloak);
  }

  if (rng.chance(0.58)) {
    addRect(svg, rng.int(9, 21), rng.int(13, 23), 1, 1, palette.accent);
  }
}

function addInstrumentSprite(svg, instrument) {
  const key = String(instrument).toLowerCase();
  const x = 15;
  const y = 17;
  const outline = "#120e09";
  const brass = "#d0a348";
  const wood = "#76512b";
  const woodDark = "#3f2616";
  const hide = "#b88145";
  const metal = "#9aa293";
  const metalDark = "#4b554d";
  const glow = "#dfff67";
  const reed = "#151b16";
  const ivory = "#eadfb3";

  switch (key) {
    case "drums":
      addIconRect(svg, x + 2, y + 2, 8, 1, outline);
      addIconRect(svg, x + 1, y + 3, 10, 2, hide);
      addIconRect(svg, x + 2, y + 5, 8, 4, wood);
      addIconRect(svg, x + 1, y + 9, 10, 1, outline);
      addIconRect(svg, x + 3, y + 5, 1, 4, woodDark);
      addIconRect(svg, x + 8, y + 5, 1, 4, woodDark);
      addIconRect(svg, x + 0, y + 0, 5, 1, brass);
      addIconRect(svg, x + 7, y + 0, 5, 1, brass);
      addIconRect(svg, x + 2, y + 1, 1, 1, brass);
      addIconRect(svg, x + 9, y + 1, 1, 1, brass);
      break;

    case "woods":
      addIconRect(svg, x + 1, y + 4, 10, 1, outline);
      addIconRect(svg, x + 2, y + 5, 8, 2, wood);
      addIconRect(svg, x + 3, y + 7, 6, 2, "#956f35");
      addIconRect(svg, x + 4, y + 9, 4, 1, woodDark);
      addIconRect(svg, x + 0, y + 1, 5, 1, brass);
      addIconRect(svg, x + 7, y + 1, 5, 1, brass);
      addIconRect(svg, x + 3, y + 2, 1, 2, brass);
      addIconRect(svg, x + 8, y + 2, 1, 2, brass);
      break;

    case "cans":
      addIconRect(svg, x + 3, y + 1, 6, 1, outline);
      addIconRect(svg, x + 2, y + 2, 8, 2, metal);
      addIconRect(svg, x + 2, y + 4, 8, 6, metalDark);
      addIconRect(svg, x + 3, y + 4, 2, 5, metal);
      addIconRect(svg, x + 2, y + 10, 8, 1, outline);
      addIconRect(svg, x + 6, y + 5, 3, 1, brass);
      addIconRect(svg, x + 6, y + 7, 3, 1, brass);
      break;

    case "tabla":
      addIconRect(svg, x + 1, y + 4, 5, 2, hide);
      addIconRect(svg, x + 7, y + 3, 4, 2, hide);
      addIconRect(svg, x + 1, y + 6, 5, 4, wood);
      addIconRect(svg, x + 7, y + 5, 4, 5, woodDark);
      addIconRect(svg, x + 3, y + 5, 1, 1, outline);
      addIconRect(svg, x + 8, y + 4, 1, 1, outline);
      addIconRect(svg, x + 0, y + 10, 12, 1, outline);
      break;

    case "harp":
      addIconRect(svg, x + 2, y, 2, 10, brass);
      addIconRect(svg, x + 4, y + 1, 5, 1, brass);
      addIconRect(svg, x + 8, y + 2, 2, 8, brass);
      addIconRect(svg, x + 3, y + 10, 8, 1, outline);
      addIconRect(svg, x + 4, y + 2, 1, 8, ivory);
      addIconRect(svg, x + 6, y + 2, 1, 8, ivory);
      addIconRect(svg, x + 8, y + 3, 1, 7, ivory);
      break;

    case "bass":
      addIconRect(svg, x + 6, y + 0, 1, 7, woodDark);
      addIconRect(svg, x + 5, y + 1, 3, 1, wood);
      addIconRect(svg, x + 3, y + 6, 6, 5, wood);
      addIconRect(svg, x + 2, y + 8, 8, 3, woodDark);
      addIconRect(svg, x + 5, y + 5, 3, 5, "#a66735");
      addIconRect(svg, x + 6, y + 1, 1, 9, ivory);
      break;

    case "pads":
      addIconRect(svg, x + 1, y + 3, 10, 7, outline);
      addIconRect(svg, x + 2, y + 4, 8, 5, "#203126");
      addIconRect(svg, x + 3, y + 5, 2, 2, glow);
      addIconRect(svg, x + 6, y + 5, 2, 2, "#86a94b");
      addIconRect(svg, x + 8, y + 7, 1, 1, brass);
      addIconRect(svg, x + 2, y + 10, 8, 1, metalDark);
      break;

    case "synth":
      addIconRect(svg, x + 1, y + 3, 10, 6, outline);
      addIconRect(svg, x + 2, y + 4, 8, 2, "#243c2f");
      addIconRect(svg, x + 2, y + 6, 8, 3, ivory);
      addIconRect(svg, x + 3, y + 6, 1, 2, outline);
      addIconRect(svg, x + 6, y + 6, 1, 2, outline);
      addIconRect(svg, x + 9, y + 6, 1, 2, outline);
      addIconRect(svg, x + 3, y + 4, 1, 1, glow);
      addIconRect(svg, x + 8, y + 4, 1, 1, brass);
      break;

    case "clarinet":
      addIconRect(svg, x + 2, y + 2, 2, 2, reed);
      addIconRect(svg, x + 4, y + 4, 2, 2, reed);
      addIconRect(svg, x + 6, y + 6, 2, 2, reed);
      addIconRect(svg, x + 8, y + 8, 2, 2, reed);
      addIconRect(svg, x + 3, y + 1, 2, 1, brass);
      addIconRect(svg, x + 5, y + 5, 1, 1, brass);
      addIconRect(svg, x + 7, y + 7, 1, 1, brass);
      addIconRect(svg, x + 9, y + 10, 3, 1, brass);
      break;

    default:
      addIconRect(svg, x + 2, y + 3, 8, 7, outline);
      addIconRect(svg, x + 3, y + 4, 6, 5, brass);
      break;
  }
}

function addShadow(svg, palette) {
  addRect(svg, 8, 29, 16, 2, palette.mud);
  addRect(svg, 11, 28, 10, 1, "#11160f");
}

function paintSymmetricRow(svg, y, left, right, fill) {
  addRect(svg, left, y, right - left + 1, 1, fill);
}

function addRect(svg, x, y, width, height, fill, className = "") {
  const rect = document.createElementNS(SVG_NS, "rect");
  rect.setAttribute("x", String(x));
  rect.setAttribute("y", String(y));
  rect.setAttribute("width", String(width));
  rect.setAttribute("height", String(height));
  rect.setAttribute("fill", fill);
  if (className) {
    rect.classList.add(...className.split(/\s+/));
  }
  svg.append(rect);
}

function addIconRect(svg, x, y, width, height, fill) {
  addRect(svg, x, y, width, height, fill, "sprite-instrument");
}
