import { deriveSeed, Rng } from "./rng.js";

const ONSETS = [
  "",
  "b",
  "bl",
  "br",
  "d",
  "dr",
  "f",
  "fl",
  "g",
  "gh",
  "gl",
  "gr",
  "h",
  "k",
  "kr",
  "l",
  "m",
  "n",
  "p",
  "r",
  "s",
  "sk",
  "sl",
  "sn",
  "th",
  "v",
  "vr",
  "z",
  "zh"
];

const NUCLEI = [
  "a",
  "e",
  "i",
  "o",
  "u",
  "ae",
  "ai",
  "oa",
  "oo",
  "ul",
  "ur",
  "og",
  "om",
  "un"
];

const CODAS = [
  "",
  "b",
  "d",
  "g",
  "k",
  "l",
  "m",
  "n",
  "r",
  "sh",
  "th",
  "x",
  "z",
  "rg",
  "lk",
  "nd",
  "ng"
];

const SWAMP_FRAGMENTS = [
  "bog",
  "fen",
  "mire",
  "mud",
  "murk",
  "ooz",
  "peat",
  "rot",
  "silt",
  "slom"
];

const ENDINGS = ["", "g", "k", "m", "n", "ok", "ug", "ul", "um", "ith"];

export function generateSpriteNames(seed, count = 4) {
  const names = [];

  for (let index = 0; index < count; index += 1) {
    let name = "";
    for (let attempt = 0; attempt < 64; attempt += 1) {
      const memberSeed = deriveSeed(seed, 0x7a4en + BigInt(index * 131 + attempt * 17));
      name = generateName(new Rng(memberSeed.value));
      if (!names.includes(name)) {
        break;
      }
    }
    names.push(name);
  }

  return names;
}

function generateName(rng) {
  for (let attempt = 0; attempt < 48; attempt += 1) {
    const syllableCount = rng.chance(0.62) ? 2 : rng.int(1, 3);
    const parts = [];

    if (rng.chance(0.38)) {
      parts.push(rng.pick(SWAMP_FRAGMENTS));
    }

    while (parts.length < syllableCount) {
      parts.push(makeSyllable(rng));
    }

    if (rng.chance(0.28)) {
      parts.push(rng.pick(ENDINGS));
    }

    const compact = compactName(parts.join(""));
    if (compact.length >= 3 && compact.length <= 10) {
      return capitalize(compact);
    }
  }

  return capitalize(compactName(`${rng.pick(SWAMP_FRAGMENTS)}${makeSyllable(rng)}`).slice(0, 10));
}

function makeSyllable(rng) {
  return `${rng.pick(ONSETS)}${rng.pick(NUCLEI)}${rng.pick(CODAS)}`;
}

function compactName(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z]/g, "")
    .replace(/([aeiou])\1{2,}/g, "$1$1")
    .replace(/([bcdfghjklmnpqrstvwxyz])\1{2,}/g, "$1$1");
}

function capitalize(value) {
  return `${value.charAt(0).toUpperCase()}${value.slice(1)}`;
}
