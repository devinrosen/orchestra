<script lang="ts">
  import { onMount } from "svelte";
  import { playerStore } from "../stores/player.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let mode = $state<"waveform" | "spectrum" | "radial">("spectrum");
  let canvas: HTMLCanvasElement;
  let animationId: number;
  let dataArray: Uint8Array<ArrayBuffer> | null = null;
  let accentColor = "";

  function resizeCanvas() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * devicePixelRatio;
    canvas.height = rect.height * devicePixelRatio;
    const ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.scale(devicePixelRatio, devicePixelRatio);
    }
  }

  function drawWaveform(ctx: CanvasRenderingContext2D, analyser: AnalyserNode, width: number, height: number) {
    analyser.getByteTimeDomainData(dataArray!);
    ctx.clearRect(0, 0, width, height);
    ctx.beginPath();
    ctx.strokeStyle = accentColor;
    ctx.lineWidth = 2;
    const len = dataArray!.length;
    for (let i = 0; i < len; i++) {
      const x = (i / len) * width;
      const y = (dataArray![i] / 255) * height;
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
  }

  function drawSpectrum(ctx: CanvasRenderingContext2D, analyser: AnalyserNode, width: number, height: number) {
    analyser.getByteFrequencyData(dataArray!);
    ctx.clearRect(0, 0, width, height);
    const numBars = Math.min(analyser.frequencyBinCount, 128);
    const barWidth = width / numBars;
    for (let i = 0; i < numBars; i++) {
      const barHeight = (dataArray![i] / 255) * height;
      const x = i * barWidth;
      const intensity = dataArray![i] / 255;
      ctx.fillStyle = `rgba(${hexToRgb(accentColor)}, ${0.3 + intensity * 0.7})`;
      ctx.fillRect(x, height - barHeight, barWidth - 1, barHeight);
    }
  }

  function drawRadial(ctx: CanvasRenderingContext2D, analyser: AnalyserNode, width: number, height: number) {
    analyser.getByteFrequencyData(dataArray!);
    ctx.clearRect(0, 0, width, height);
    const cx = width / 2;
    const cy = height / 2;
    const baseRadius = Math.min(width, height) * 0.2;
    const numBars = Math.min(analyser.frequencyBinCount, 128);
    ctx.strokeStyle = accentColor;
    ctx.lineWidth = 2;
    for (let i = 0; i < numBars; i++) {
      const angle = (i / numBars) * 2 * Math.PI;
      const magnitude = (dataArray![i] / 255) * (Math.min(width, height) * 0.25);
      const cosA = Math.cos(angle);
      const sinA = Math.sin(angle);
      // Outward bar
      ctx.beginPath();
      ctx.moveTo(cx + cosA * baseRadius, cy + sinA * baseRadius);
      ctx.lineTo(cx + cosA * (baseRadius + magnitude), cy + sinA * (baseRadius + magnitude));
      ctx.stroke();
      // Inward bar (mirror)
      const inward = magnitude * 0.4;
      ctx.beginPath();
      ctx.moveTo(cx + cosA * baseRadius, cy + sinA * baseRadius);
      ctx.lineTo(cx + cosA * (baseRadius - inward), cy + sinA * (baseRadius - inward));
      ctx.stroke();
    }
  }

  function hexToRgb(hex: string): string {
    const cleaned = hex.replace("#", "");
    if (cleaned.length < 6) return "100, 149, 237";
    const r = parseInt(cleaned.substring(0, 2), 16);
    const g = parseInt(cleaned.substring(2, 4), 16);
    const b = parseInt(cleaned.substring(4, 6), 16);
    return `${r}, ${g}, ${b}`;
  }

  function draw() {
    animationId = requestAnimationFrame(draw);
    const analyser = playerStore.getAnalyser();
    if (!analyser || !canvas) return;

    if (!dataArray || dataArray.length !== analyser.frequencyBinCount) {
      dataArray = new Uint8Array(analyser.frequencyBinCount) as Uint8Array<ArrayBuffer>;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const width = rect.width;
    const height = rect.height;

    if (mode === "waveform") {
      drawWaveform(ctx, analyser, width, height);
    } else if (mode === "spectrum") {
      drawSpectrum(ctx, analyser, width, height);
    } else {
      drawRadial(ctx, analyser, width, height);
    }
  }

  onMount(() => {
    accentColor = getComputedStyle(canvas).getPropertyValue("--accent").trim();
    resizeCanvas();

    const observer = new ResizeObserver(() => resizeCanvas());
    observer.observe(canvas);

    animationId = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animationId);
      observer.disconnect();
    };
  });
</script>

<div class="visualizer-panel">
  <div class="visualizer-header">
    <div class="mode-buttons">
      <button class="mode-btn" class:active={mode === "waveform"} onclick={() => mode = "waveform"}>Waveform</button>
      <button class="mode-btn" class:active={mode === "spectrum"} onclick={() => mode = "spectrum"}>Spectrum</button>
      <button class="mode-btn" class:active={mode === "radial"} onclick={() => mode = "radial"}>Radial</button>
    </div>
    <button class="close-btn" onclick={onClose}>x</button>
  </div>
  <canvas bind:this={canvas} class="visualizer-canvas"></canvas>
</div>

<style>
  .visualizer-panel {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    height: 180px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .visualizer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .mode-buttons {
    display: flex;
    gap: 4px;
  }

  .mode-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: var(--radius);
    cursor: pointer;
  }

  .mode-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .mode-btn.active {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    padding: 2px 6px;
    cursor: pointer;
    border-radius: var(--radius);
  }

  .close-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .visualizer-canvas {
    width: 100%;
    flex: 1;
    display: block;
    min-height: 0;
  }
</style>
