<script lang="ts">
  interface Props {
    pattern: string;
    onPatternChange: (pattern: string) => void;
  }

  let { pattern, onPatternChange }: Props = $props();

  let inputEl = $state<HTMLInputElement | null>(null);

  const PLACEHOLDER = "{artist}/{album}/{track_number} - {title}.{ext}";

  const TOKENS = [
    { token: "{artist}", label: "Artist" },
    { token: "{album_artist}", label: "Album Artist" },
    { token: "{album}", label: "Album" },
    { token: "{title}", label: "Title" },
    { token: "{track_number}", label: "Track #" },
    { token: "{disc_number}", label: "Disc #" },
    { token: "{year}", label: "Year" },
    { token: "{genre}", label: "Genre" },
    { token: "{ext}", label: "Ext" },
  ];

  const EXAMPLE_TRACK = {
    artist: "Pink Floyd",
    album_artist: "Pink Floyd",
    album: "The Dark Side of the Moon",
    title: "Time",
    track_number: "03",
    disc_number: "1",
    year: "1973",
    genre: "Progressive Rock",
    ext: "flac",
  };

  function insertToken(token: string) {
    if (!inputEl) {
      onPatternChange(pattern + token);
      return;
    }
    const start = inputEl.selectionStart ?? pattern.length;
    const end = inputEl.selectionEnd ?? pattern.length;
    const next = pattern.slice(0, start) + token + pattern.slice(end);
    onPatternChange(next);
    // Restore cursor after token
    requestAnimationFrame(() => {
      if (inputEl) {
        const pos = start + token.length;
        inputEl.setSelectionRange(pos, pos);
        inputEl.focus();
      }
    });
  }

  function previewPattern(pat: string): string {
    let result = pat;
    for (const [key, value] of Object.entries(EXAMPLE_TRACK)) {
      result = result.replaceAll(`{${key}}`, value);
    }
    return result;
  }

  const isValid = $derived(
    pattern.includes("{ext}") && pattern.trim().length > 0,
  );
</script>

<div class="pattern-editor">
  <label class="label" for="pattern-input">Naming Pattern</label>
  <input
    id="pattern-input"
    class="pattern-input"
    type="text"
    value={pattern}
    bind:this={inputEl}
    oninput={(e) => onPatternChange((e.target as HTMLInputElement).value)}
    placeholder={PLACEHOLDER}
    spellcheck="false"
  />

  {#if !isValid && pattern.trim().length > 0}
    <p class="validation-error">Pattern must include <code>{"{ext}"}</code> to preserve the file extension.</p>
  {/if}

  <div class="token-bar">
    {#each TOKENS as { token, label }}
      <button
        class="token-btn"
        type="button"
        title="Insert {token}"
        onclick={() => insertToken(token)}
      >
        {label}
      </button>
    {/each}
  </div>

  <div class="preview">
    <span class="preview-label">Preview:</span>
    <span class="preview-value">{previewPattern(pattern) || "—"}</span>
  </div>
</div>

<style>
  .pattern-editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .label {
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .pattern-input {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    font-size: 13px;
    font-family: monospace;
  }

  .pattern-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .validation-error {
    font-size: 12px;
    color: var(--error, #e55);
    margin: 0;
  }

  .token-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .token-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--accent);
    padding: 4px 8px;
    font-size: 12px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .token-btn:hover {
    background: var(--bg-secondary);
  }

  .preview {
    font-size: 12px;
    color: var(--text-secondary);
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .preview-label {
    font-weight: 500;
  }

  .preview-value {
    font-family: monospace;
    color: var(--text-primary);
    word-break: break-all;
  }
</style>
