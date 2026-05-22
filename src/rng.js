const MASK_64 = 0xffff_ffff_ffff_ffffn;
const SPLITMIX_INCREMENT = 0x9e37_79b9_7f4a_7c15n;

export function createSeed() {
  const values = new Uint32Array(2);
  if (globalThis.crypto?.getRandomValues) {
    globalThis.crypto.getRandomValues(values);
  } else {
    values[0] = Math.floor(Math.random() * 0x1_0000_0000);
    values[1] = Math.floor(Math.random() * 0x1_0000_0000);
  }

  if (values[0] === 0 && values[1] === 0) {
    values[1] = 1;
  }

  return {
    hi: values[0] >>> 0,
    lo: values[1] >>> 0,
    value: (BigInt(values[0]) << 32n) | BigInt(values[1])
  };
}

export function formatSeed(seed) {
  return `0x${seed.value.toString(16).padStart(16, "0")}`;
}

export function deriveSeed(seed, salt) {
  const saltValue = BigInt.asUintN(64, BigInt(salt));
  const mixed = mix64(seed.value ^ (saltValue * SPLITMIX_INCREMENT));
  return {
    hi: Number((mixed >> 32n) & 0xffff_ffffn) >>> 0,
    lo: Number(mixed & 0xffff_ffffn) >>> 0,
    value: mixed
  };
}

export class Rng {
  constructor(seedValue) {
    this.state = BigInt.asUintN(64, seedValue || 0xa5a5_1f2e_3d4c_5b6an);
  }

  nextBigInt() {
    this.state = BigInt.asUintN(64, this.state + SPLITMIX_INCREMENT);
    return mix64(this.state);
  }

  nextU32() {
    return Number((this.nextBigInt() >> 32n) & 0xffff_ffffn) >>> 0;
  }

  nextFloat() {
    return (this.nextU32() >>> 8) / 0x1_000000;
  }

  chance(probability) {
    return this.nextFloat() < Math.max(0, Math.min(1, probability));
  }

  int(min, maxInclusive) {
    const span = maxInclusive - min + 1;
    return min + (this.nextU32() % span);
  }

  pick(values) {
    return values[this.int(0, values.length - 1)];
  }
}

function mix64(input) {
  let value = BigInt.asUintN(64, input);
  value = BigInt.asUintN(64, (value ^ (value >> 30n)) * 0xbf58_476d_1ce4_e5b9n);
  value = BigInt.asUintN(64, (value ^ (value >> 27n)) * 0x94d0_49bb_1331_11ebn);
  return BigInt.asUintN(64, value ^ (value >> 31n));
}
