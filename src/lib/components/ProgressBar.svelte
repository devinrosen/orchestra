<script lang="ts">
  let {
    value = 0,
    max = 100,
    label = "",
    showPercentage = true,
  }: {
    value: number;
    max: number;
    label?: string;
    showPercentage?: boolean;
  } = $props();

  let percentage = $derived(max > 0 ? Math.round((value / max) * 100) : 0);
</script>

<div class="progress-container">
  {#if label}
    <div class="progress-label">
      <span>{label}</span>
      {#if showPercentage}<span>{percentage}%</span>{/if}
    </div>
  {/if}
  <div class="progress-track">
    <div class="progress-fill" style="width: {percentage}%"></div>
  </div>
</div>

<style>
  .progress-container {
    width: 100%;
  }

  .progress-label {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .progress-track {
    width: 100%;
    height: 8px;
    background-color: var(--bg-secondary);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: var(--accent);
    border-radius: 4px;
    transition: width 0.2s ease;
  }
</style>
