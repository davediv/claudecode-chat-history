<script lang="ts">
  /**
   * SearchInput component with debouncing and keyboard shortcuts.
   *
   * Features:
   * - Debounced input (300ms) before triggering search
   * - Keyboard shortcut: `/` focuses input from anywhere
   * - `Escape` clears input and removes focus
   * - Shows loading indicator during search
   * - Clear button (X) when text present
   */
  import { onMount, onDestroy } from "svelte";

  interface Props {
    /** Current search query (bindable) */
    value?: string;
    /** Placeholder text */
    placeholder?: string;
    /** Whether search is currently in progress */
    isSearching?: boolean;
    /** Debounce delay in milliseconds */
    debounceMs?: number;
    /** Handler called after debounce with the search query */
    onSearch?: (query: string) => void;
    /** Handler called immediately on input change */
    onInput?: (query: string) => void;
  }

  let {
    value = $bindable(""),
    placeholder = "Search conversations...",
    isSearching = false,
    debounceMs = 300,
    onSearch,
    onInput,
  }: Props = $props();

  let inputElement: HTMLInputElement | undefined = $state();
  let debounceTimeout: ReturnType<typeof setTimeout> | null = null;

  /**
   * Handle input changes with debouncing.
   */
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onInput?.(value);

    // Clear any pending debounce
    if (debounceTimeout) {
      clearTimeout(debounceTimeout);
    }

    // Set up new debounce
    debounceTimeout = setTimeout(() => {
      onSearch?.(value);
      debounceTimeout = null;
    }, debounceMs);
  }

  /**
   * Handle keyboard events on the input.
   */
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      // Clear input and blur
      value = "";
      onInput?.("");
      onSearch?.("");
      inputElement?.blur();
      event.preventDefault();
    }
  }

  /**
   * Clear the search input.
   */
  function handleClear() {
    value = "";
    onInput?.("");
    onSearch?.("");
    inputElement?.focus();
  }

  /**
   * Global keyboard shortcut handler.
   * `/` focuses the search input from anywhere.
   */
  function handleGlobalKeydown(event: KeyboardEvent) {
    // Ignore if typing in an input, textarea, or contenteditable
    const target = event.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
      return;
    }

    // Focus search on `/` key
    if (event.key === "/") {
      event.preventDefault();
      inputElement?.focus();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleGlobalKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleGlobalKeydown);
    if (debounceTimeout) {
      clearTimeout(debounceTimeout);
    }
  });
</script>

<div class="search-container">
  <!-- Search icon or loading spinner -->
  {#if isSearching}
    <svg
      class="search-icon loading"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="10" stroke-dasharray="32" stroke-dashoffset="32">
        <animate
          attributeName="stroke-dashoffset"
          values="32;0"
          dur="1s"
          repeatCount="indefinite"
        />
      </circle>
    </svg>
  {:else}
    <svg
      class="search-icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      aria-hidden="true"
    >
      <circle cx="11" cy="11" r="8"></circle>
      <path d="m21 21-4.35-4.35"></path>
    </svg>
  {/if}

  <input
    bind:this={inputElement}
    type="text"
    class="search-input"
    {placeholder}
    {value}
    oninput={handleInput}
    onkeydown={handleKeydown}
    aria-label="Search conversations"
    aria-busy={isSearching}
  />

  <!-- Keyboard shortcut hint -->
  {#if !value}
    <kbd class="shortcut-hint" aria-hidden="true">/</kbd>
  {/if}

  <!-- Clear button -->
  {#if value}
    <button class="clear-button" aria-label="Clear search" onclick={handleClear} type="button">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12"></path>
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-container {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }

  .search-icon {
    position: absolute;
    left: 0.75rem;
    width: 1rem;
    height: 1rem;
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .search-icon.loading {
    color: var(--color-accent);
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 2.5rem 0.5rem 2.25rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 0.875rem;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .search-input::placeholder {
    color: var(--color-text-muted);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px rgba(var(--color-accent-rgb, 99, 102, 241), 0.1);
  }

  .shortcut-hint {
    position: absolute;
    right: 0.625rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.25rem;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 0.6875rem;
    font-weight: 500;
    pointer-events: none;
  }

  .clear-button {
    position: absolute;
    right: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .clear-button:hover {
    background-color: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .clear-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .clear-button svg {
    width: 0.875rem;
    height: 0.875rem;
  }
</style>
