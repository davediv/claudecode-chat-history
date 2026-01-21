<script lang="ts">
  /**
   * TagFilter component for filtering conversations by tags.
   *
   * Features:
   * - Dropdown with all available tags
   * - Multi-select for filtering by multiple tags (AND logic)
   * - Shows tag usage counts
   * - Clear all selected tags
   */
  import type { TagInfo } from "$lib/types";

  interface Props {
    /** All available tags with counts */
    allTags?: TagInfo[];
    /** Currently selected tags for filtering */
    selectedTags?: string[];
    /** Handler for tag selection changes */
    onTagsChange?: (tags: string[]) => void;
  }

  let { allTags = [], selectedTags = [], onTagsChange }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef: HTMLDivElement | undefined = $state();

  function toggleTag(tag: string) {
    if (selectedTags.includes(tag)) {
      onTagsChange?.(selectedTags.filter((t) => t !== tag));
    } else {
      onTagsChange?.([...selectedTags, tag].sort());
    }
  }

  function clearAll() {
    onTagsChange?.([]);
  }

  function handleButtonKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      isOpen = !isOpen;
    } else if (event.key === "Escape" && isOpen) {
      isOpen = false;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  // Close on click outside
  $effect(() => {
    if (isOpen) {
      document.addEventListener("click", handleClickOutside);
      return () => document.removeEventListener("click", handleClickOutside);
    }
  });
</script>

<div class="tag-filter" bind:this={dropdownRef}>
  <button
    type="button"
    class="filter-button"
    class:active={selectedTags.length > 0}
    onclick={() => (isOpen = !isOpen)}
    onkeydown={handleButtonKeydown}
    aria-expanded={isOpen}
    aria-haspopup="listbox"
    aria-label="Filter by tags"
    title="Filter by tags"
  >
    <svg class="tag-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z"></path>
      <line x1="7" y1="7" x2="7.01" y2="7"></line>
    </svg>
    {#if selectedTags.length > 0}
      <span class="filter-count">{selectedTags.length}</span>
    {/if}
    <svg
      class="chevron-icon"
      class:rotated={isOpen}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>

  {#if isOpen}
    <div class="dropdown-menu" role="listbox" aria-multiselectable="true">
      {#if allTags.length === 0}
        <div class="empty-state">No tags yet</div>
      {:else}
        <div class="dropdown-header">
          <span class="header-title">Filter by tags</span>
          {#if selectedTags.length > 0}
            <button type="button" class="clear-button" onclick={clearAll}> Clear all </button>
          {/if}
        </div>
        <ul class="tag-list">
          {#each allTags as tagInfo (tagInfo.tag)}
            <li>
              <label class="tag-option">
                <input
                  type="checkbox"
                  checked={selectedTags.includes(tagInfo.tag)}
                  onchange={() => toggleTag(tagInfo.tag)}
                />
                <span class="tag-name">{tagInfo.tag}</span>
                <span class="tag-count">{tagInfo.count}</span>
              </label>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tag-filter {
    position: relative;
  }

  .filter-button {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.5rem;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .filter-button:hover {
    background: var(--color-bg-tertiary);
  }

  .filter-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .filter-button.active {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .tag-icon {
    width: 1rem;
    height: 1rem;
  }

  .chevron-icon {
    width: 0.875rem;
    height: 0.875rem;
    transition: transform 0.15s ease;
  }

  .chevron-icon.rotated {
    transform: rotate(180deg);
  }

  .filter-count {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 1.125rem;
    height: 1.125rem;
    padding: 0 0.25rem;
    background: var(--color-accent);
    border-radius: 10px;
    color: white;
    font-size: 0.6875rem;
    font-weight: 600;
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.25rem);
    right: 0;
    min-width: 200px;
    max-width: 280px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    z-index: 100;
    overflow: hidden;
  }

  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.625rem 0.75rem;
    border-bottom: 1px solid var(--color-border);
  }

  .header-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .clear-button {
    padding: 0.125rem 0.375rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-accent);
    font-size: 0.75rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .clear-button:hover {
    background: var(--color-bg-tertiary);
  }

  .tag-list {
    list-style: none;
    margin: 0;
    padding: 0.25rem;
    max-height: 240px;
    overflow-y: auto;
  }

  .tag-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .tag-option:hover {
    background: var(--color-bg-tertiary);
  }

  .tag-option input[type="checkbox"] {
    width: 1rem;
    height: 1rem;
    accent-color: var(--color-accent);
    cursor: pointer;
  }

  .tag-name {
    flex: 1;
    color: var(--color-text-primary);
    font-size: 0.8125rem;
  }

  .tag-count {
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .empty-state {
    padding: 1rem;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.8125rem;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .filter-button,
    .chevron-icon,
    .clear-button,
    .tag-option {
      transition: none;
    }
  }
</style>
