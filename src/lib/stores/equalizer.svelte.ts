import { getSetting, setSetting } from "../api/commands";

export interface EqBand {
  frequency: number;
  type: BiquadFilterType;
  gain: number;
}

export interface EqPreset {
  name: string;
  gains: number[];
}

const EQ_FREQUENCIES = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

const EQ_FILTER_TYPES: BiquadFilterType[] = [
  "lowshelf",
  "peaking",
  "peaking",
  "peaking",
  "peaking",
  "peaking",
  "peaking",
  "peaking",
  "peaking",
  "highshelf",
];

const DEFAULT_BANDS: EqBand[] = EQ_FREQUENCIES.map((freq, i) => ({
  frequency: freq,
  type: EQ_FILTER_TYPES[i],
  gain: 0,
}));

export const EQ_PRESETS: EqPreset[] = [
  { name: "flat", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  { name: "bass boost", gains: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0] },
  { name: "treble boost", gains: [0, 0, 0, 0, 0, 0, 2, 4, 5, 6] },
  { name: "vocal", gains: [-2, -1, 0, 2, 4, 4, 2, 0, -1, -2] },
  { name: "rock", gains: [4, 3, 1, 0, -1, -1, 0, 2, 3, 4] },
  { name: "electronic", gains: [4, 3, 1, 0, -2, 0, 1, 3, 4, 5] },
];

class EqualizerStore {
  bands = $state<EqBand[]>(DEFAULT_BANDS.map((b) => ({ ...b })));
  activePreset = $state<string>("flat");
  enabled = $state(true);

  isFlat = $derived(this.bands.every((b) => b.gain === 0));

  private filterNodes: BiquadFilterNode[] = [];

  setBandGain(index: number, gain: number) {
    if (index < 0 || index >= this.bands.length) return;
    this.bands[index].gain = gain;
    this.activePreset = "custom";
    this.applyGains();
  }

  applyPreset(name: string) {
    const preset = EQ_PRESETS.find((p) => p.name === name);
    if (!preset) return;
    for (let i = 0; i < this.bands.length; i++) {
      this.bands[i].gain = preset.gains[i];
    }
    this.activePreset = name;
    this.applyGains();
  }

  toggleEnabled() {
    this.enabled = !this.enabled;
    this.applyGains();
  }

  connectFilters(
    ctx: AudioContext,
    source: AudioNode,
    destination: AudioNode,
  ): AudioNode {
    this.filterNodes = [];
    for (let i = 0; i < this.bands.length; i++) {
      const filter = ctx.createBiquadFilter();
      filter.type = this.bands[i].type;
      filter.frequency.value = this.bands[i].frequency;
      if (this.bands[i].type === "peaking") {
        filter.Q.value = 1.4;
      }
      this.filterNodes.push(filter);
    }

    source.connect(this.filterNodes[0]);
    for (let i = 0; i < this.filterNodes.length - 1; i++) {
      this.filterNodes[i].connect(this.filterNodes[i + 1]);
    }
    this.filterNodes[this.filterNodes.length - 1].connect(destination);

    this.applyGains();
    return this.filterNodes[this.filterNodes.length - 1];
  }

  applyGains() {
    for (let i = 0; i < this.filterNodes.length; i++) {
      this.filterNodes[i].gain.value = this.enabled ? this.bands[i].gain : 0;
    }
  }

  async loadState() {
    try {
      const [presetStr, gainsStr, enabledStr] = await Promise.all([
        getSetting("eq_preset"),
        getSetting("eq_gains"),
        getSetting("eq_enabled"),
      ]);

      if (enabledStr !== null) {
        this.enabled = enabledStr === "true";
      }

      if (gainsStr !== null) {
        const gains: number[] = JSON.parse(gainsStr);
        if (Array.isArray(gains) && gains.length === 10) {
          for (let i = 0; i < this.bands.length; i++) {
            this.bands[i].gain = gains[i];
          }
        }
      }

      if (presetStr !== null) {
        this.activePreset = presetStr;
      }

      this.applyGains();
    } catch {
      // Settings not found or invalid — use defaults
    }
  }

  async saveState() {
    try {
      const gains = this.bands.map((b) => b.gain);
      await Promise.all([
        setSetting("eq_preset", this.activePreset),
        setSetting("eq_gains", JSON.stringify(gains)),
        setSetting("eq_enabled", String(this.enabled)),
      ]);
    } catch {
      // Silently fail — non-critical persistence
    }
  }
}

export const equalizerStore = new EqualizerStore();
