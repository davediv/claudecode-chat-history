<script lang="ts">
  /**
   * TagInput component for adding and removing tags on conversations.
   *
   * Features:
   * - Display current tags as removable chips
   * - Input field for adding new tags
   * - Autocomplete dropdown from existing tags
   * - Keyboard navigation (Enter to add, Tab to select suggestion)
   */
  import type { TagInfo } from "$lib/types";

  interface Props {
    /** Current tags on this conversation */
    tags: string[];
    /** All available tags for autocomplete */
    allTags?: TagInfo[];
    /** Handler for tag changes */
    onTagsChange?: (tags: string[]) => void;
    /** Whether input is disabled */
    disabled?: boolean;
  }

  let { tags = [], allTags = [], onTagsChange, disabled = false }: Props = $props();

  let inputValue = $state("");
  let showSuggestions = $state(false);
  let selectedSuggestionIndex = $state(-1);
  let inputElement: HTMLInputElement | undefined = $state();
  let blurTimeoutId: ReturnType<typeof setTimeout> | null = null;

  // Cleanup timeout on unmount
  $effect(() => {
    return () => {
      if (blurTimeoutId) clearTimeout(blurTimeoutId);
    };
  });

  // Filter suggestions based on input and exclude already added tags
  const suggestions = $derived(() => {
    if (!inputValue.trim()) return [];
    const query = inputValue.toLowerCase().trim();
    return allTags.filter((t) => t.tag.includes(query) && !tags.includes(t.tag)).slice(0, 5);
  });

  function addTag(tag: string) {
    const normalized = tag.trim().toLowerCase();
    if (normalized && !tags.includes(normalized)) {
      const newTags = [...tags, normalized].sort();
      onTagsChange?.(newTags);
    }
    inputValue = "";
    showSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function removeTag(tag: string) {
    const newTags = tags.filter((t) => t !== tag);
    onTagsChange?.(newTags);
  }

  function handleKeydown(event: KeyboardEvent) {
    const suggestionsList = suggestions();

    if (event.key === "Enter") {
      event.preventDefault();
      if (selectedSuggestionIndex >= 0 && selectedSuggestionIndex < suggestionsList.length) {
        addTag(suggestionsList[selectedSuggestionIndex].tag);
      } else if (inputValue.trim()) {
        addTag(inputValue);
      }
    } else if (event.key === "Tab" && suggestionsList.length > 0 && showSuggestions) {
      event.preventDefault();
      const index = selectedSuggestionIndex >= 0 ? selectedSuggestionIndex : 0;
      addTag(suggestionsList[index].tag);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      if (suggestionsList.length > 0) {
        showSuggestions = true;
        selectedSuggestionIndex = Math.min(selectedSuggestionIndex + 1, suggestionsList.length - 1);
      }
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (suggestionsList.length > 0) {
        selectedSuggestionIndex = Math.max(selectedSuggestionIndex - 1, 0);
      }
    } else if (event.key === "Escape") {
      showSuggestions = false;
      selectedSuggestionIndex = -1;
    } else if (event.key === "Backspace" && !inputValue && tags.length > 0) {
      // Remove last tag on backspace when input is empty
      removeTag(tags[tags.length - 1]);
    }
  }

  function handleInput() {
    showSuggestions = true;
    selectedSuggestionIndex = -1;
  }

  function handleFocus() {
    if (inputValue.trim()) {
      showSuggestions = true;
    }
  }

  function handleBlur() {
    // Delay hiding to allow click on suggestion
    if (blurTimeoutId) clearTimeout(blurTimeoutId);
    blurTimeoutId = setTimeout(() => {
      showSuggestions = false;
      selectedSuggestionIndex = -1;
      blurTimeoutId = null;
    }, 150);
  }

  function handleTagKeydown(event: KeyboardEvent, tag: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      removeTag(tag);
    }
  }
</script>

<div class="tag-input-container">
  <div class="tags-display">
    {#each tags as tag (tag)}
      <span class="tag-chip">
        <span class="tag-text">{tag}</span>
        <button
          type="button"
          class="tag-remove"
          onclick={() => removeTag(tag)}
          onkeydown={(e) => handleTagKeydown(e, tag)}
          aria-label={`Remove tag ${tag}`}
          {disabled}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"></path>
          </svg>
        </button>
      </span>
    {/each}

    <div class="input-wrapper">
      <input
        bind:this={inputElement}
        bind:value={inputValue}
        type="text"
        class="tag-input"
        placeholder={tags.length === 0 ? "Add tags..." : ""}
        oninput={handleInput}
        onkeydown={handleKeydown}
        onfocus={handleFocus}
        onblur={handleBlur}
        {disabled}
        role="combobox"
        aria-label="Add tag"
        aria-autocomplete="list"
        aria-expanded={showSuggestions && suggestions().length > 0}
        aria-controls="tag-suggestions"
      />

      {#if showSuggestions && suggestions().length > 0}
        <ul id="tag-suggestions" class="suggestions-dropdown" role="listbox">
          {#each suggestions() as suggestion, index (suggestion.tag)}
            <li
              class="suggestion-item"
              class:selected={index === selectedSuggestionIndex}
              role="option"
              aria-selected={index === selectedSuggestionIndex}
              onmousedown={() => addTag(suggestion.tag)}
              onmouseenter={() => (selectedSuggestionIndex = index)}
            >
              <span class="suggestion-tag">{suggestion.tag}</span>
              <span class="suggestion-count">{suggestion.count}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>

<style>
  .tag-input-container {
    width: 100%;
  }

  .tags-display {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    min-height: 2rem;
    transition: border-color 0.15s ease;
  }

  .tags-display:focus-within {
    border-color: var(--color-accent);
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.375rem;
    background: var(--color-bg-tertiary);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .tag-text {
    line-height: 1.4;
  }

  .tag-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 2px;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      color 0.15s ease,
      background-color 0.15s ease;
  }

  .tag-remove:hover:not(:disabled) {
    color: var(--color-text-primary);
    background: var(--color-bg-secondary);
  }

  .tag-remove:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .tag-remove svg {
    width: 0.75rem;
    height: 0.75rem;
  }

  .input-wrapper {
    position: relative;
    flex: 1;
    min-width: 60px;
  }

  .tag-input {
    width: 100%;
    padding: 0.125rem 0.25rem;
    background: transparent;
    border: none;
    font-size: 0.75rem;
    color: var(--color-text-primary);
    outline: none;
  }

  .tag-input::placeholder {
    color: var(--color-text-muted);
  }

  .tag-input:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .suggestions-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin: 0.25rem 0 0 0;
    padding: 0.25rem;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    list-style: none;
    z-index: 100;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.375rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: var(--color-bg-tertiary);
  }

  .suggestion-tag {
    color: var(--color-text-primary);
  }

  .suggestion-count {
    color: var(--color-text-muted);
    font-size: 0.675rem;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .tags-display,
    .tag-remove,
    .suggestion-item {
      transition: none;
    }
  }
</style>
