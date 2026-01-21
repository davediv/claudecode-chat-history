<script lang="ts">
  /**
   * Reusable filter dropdown component.
   *
   * Features:
   * - Keyboard accessible: arrow keys, Enter, Escape
   * - Shows checkmark next to selected option
   * - Scrollable when many options
   * - Supports optional "All" option
   */
  import { onMount, onDestroy } from "svelte";

  interface Option {
    value: string;
    label: string;
    count?: number;
  }

  interface Props {
    /** Available options */
    options: Option[];
    /** Currently selected value (empty string for "All") */
    selected?: string;
    /** Placeholder text when no selection */
    placeholder?: string;
    /** Label for the "All" option */
    allLabel?: string;
    /** Show option counts */
    showCounts?: boolean;
    /** Handler for selection changes */
    onChange?: (value: string) => void;
    /** Accessible label for the dropdown */
    ariaLabel?: string;
  }

  let {
    options,
    selected = "",
    placeholder = "Select...",
    allLabel = "All",
    showCounts = false,
    onChange,
    ariaLabel = "Filter dropdown",
  }: Props = $props();

  let isOpen = $state(false);
  let focusedIndex = $state(-1);
  let dropdownRef: HTMLDivElement | undefined = $state();
  let buttonRef: HTMLButtonElement | undefined = $state();
  let listRef: HTMLDivElement | undefined = $state();

  // All options including the "All" option
  const allOptions = $derived([{ value: "", label: allLabel }, ...options]);

  // Get display label for current selection
  const displayLabel = $derived(() => {
    if (!selected) return placeholder;
    const option = options.find((o) => o.value === selected);
    return option?.label ?? selected;
  });

  function toggle() {
    isOpen = !isOpen;
    if (isOpen) {
      // Focus the current selection
      const idx = allOptions.findIndex((o) => o.value === selected);
      focusedIndex = idx >= 0 ? idx : 0;
    }
  }

  function close() {
    isOpen = false;
    focusedIndex = -1;
    buttonRef?.focus();
  }

  function selectOption(value: string) {
    onChange?.(value);
    close();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!isOpen) {
      if (event.key === "Enter" || event.key === " " || event.key === "ArrowDown") {
        event.preventDefault();
        isOpen = true;
        focusedIndex = allOptions.findIndex((o) => o.value === selected);
        if (focusedIndex < 0) focusedIndex = 0;
      }
      return;
    }

    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, allOptions.length - 1);
        scrollToFocused();
        break;
      case "ArrowUp":
        event.preventDefault();
        focusedIndex = Math.max(focusedIndex - 1, 0);
        scrollToFocused();
        break;
      case "Home":
        event.preventDefault();
        focusedIndex = 0;
        scrollToFocused();
        break;
      case "End":
        event.preventDefault();
        focusedIndex = allOptions.length - 1;
        scrollToFocused();
        break;
      case "Enter":
      case " ":
        event.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < allOptions.length) {
          selectOption(allOptions[focusedIndex].value);
        }
        break;
      case "Escape":
        event.preventDefault();
        close();
        break;
      case "Tab":
        close();
        break;
    }
  }

  function scrollToFocused() {
    if (!listRef) return;
    const focusedEl = listRef.querySelector(`[data-index="${focusedIndex}"]`);
    if (focusedEl) {
      focusedEl.scrollIntoView({ block: "nearest" });
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      close();
    }
  }

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
  });
</script>

<div class="dropdown" bind:this={dropdownRef}>
  <button
    bind:this={buttonRef}
    type="button"
    class="dropdown-button"
    aria-haspopup="listbox"
    aria-expanded={isOpen}
    aria-label={ariaLabel}
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <span class="button-label">{displayLabel()}</span>
    <svg
      class="dropdown-icon"
      class:open={isOpen}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>

  {#if isOpen}
    <div
      bind:this={listRef}
      class="dropdown-list"
      role="listbox"
      tabindex="-1"
      aria-activedescendant={focusedIndex >= 0 ? `option-${focusedIndex}` : undefined}
    >
      {#each allOptions as option, index (option.value)}
        <button
          id="option-{index}"
          type="button"
          class="dropdown-option"
          class:selected={option.value === selected}
          class:focused={index === focusedIndex}
          role="option"
          aria-selected={option.value === selected}
          data-index={index}
          onclick={() => selectOption(option.value)}
          onmouseenter={() => (focusedIndex = index)}
        >
          <span class="option-label">{option.label}</span>
          {#if showCounts && option.count !== undefined}
            <span class="option-count">{option.count}</span>
          {/if}
          {#if option.value === selected}
            <svg
              class="check-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
    display: inline-block;
  }

  .dropdown-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    min-width: 140px;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease;
  }

  .dropdown-button:hover {
    background-color: var(--color-border-light);
    border-color: var(--color-accent);
  }

  .dropdown-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .button-label {
    flex: 1;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dropdown-icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    transition: transform 0.15s ease;
  }

  .dropdown-icon.open {
    transform: rotate(180deg);
  }

  .dropdown-list {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    min-width: 100%;
    max-height: 240px;
    margin-top: 4px;
    padding: 4px 0;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow-y: auto;
  }

  .dropdown-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .dropdown-option:hover,
  .dropdown-option.focused {
    background-color: var(--color-bg-tertiary);
  }

  .dropdown-option.selected {
    color: var(--color-accent);
    font-weight: 500;
  }

  .option-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .option-count {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    padding: 0.125rem 0.375rem;
    background-color: var(--color-bg-tertiary);
    border-radius: 4px;
  }

  .check-icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    color: var(--color-accent);
  }
</style>
