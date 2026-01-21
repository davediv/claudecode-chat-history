<script lang="ts">
  /**
   * DateRangePicker component for filtering conversations by date.
   *
   * Features:
   * - Quick presets: Today, Last 7 days, Last 30 days, All time
   * - Custom date range with native date pickers
   * - Clear filter button
   * - Updates filters store on selection
   */
  import { filtersStore } from "$lib/stores";
  import { SvelteDate } from "svelte/reactivity";

  interface Props {
    /** Optional handler called when date range changes */
    onChange?: (start: string | null, end: string | null) => void;
  }

  let { onChange }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef: HTMLDivElement | undefined = $state();
  let buttonRef: HTMLButtonElement | undefined = $state();

  // Get current filter values from store
  const startDate = $derived(filtersStore.dateStart ?? "");
  const endDate = $derived(filtersStore.dateEnd ?? "");

  /**
   * Get display label based on current selection.
   * Computed each time since it depends on current time for preset matching.
   */
  function getDisplayLabel(): string {
    if (!filtersStore.dateStart && !filtersStore.dateEnd) {
      return "All time";
    }

    // Check if matches a preset
    const now = Date.now();
    const today = new SvelteDate(now);
    today.setHours(0, 0, 0, 0);
    const todayStr = formatDateForInput(today);

    if (filtersStore.dateStart === todayStr && filtersStore.dateEnd === todayStr) {
      return "Today";
    }

    const last7 = new SvelteDate(today.getTime());
    last7.setDate(last7.getDate() - 6);
    if (filtersStore.dateStart === formatDateForInput(last7) && filtersStore.dateEnd === todayStr) {
      return "Last 7 days";
    }

    const last30 = new SvelteDate(today.getTime());
    last30.setDate(last30.getDate() - 29);
    if (
      filtersStore.dateStart === formatDateForInput(last30) &&
      filtersStore.dateEnd === todayStr
    ) {
      return "Last 30 days";
    }

    // Custom range
    if (filtersStore.dateStart && filtersStore.dateEnd) {
      return `${formatDisplayDate(filtersStore.dateStart)} - ${formatDisplayDate(filtersStore.dateEnd)}`;
    }
    if (filtersStore.dateStart) {
      return `From ${formatDisplayDate(filtersStore.dateStart)}`;
    }
    if (filtersStore.dateEnd) {
      return `Until ${formatDisplayDate(filtersStore.dateEnd)}`;
    }

    return "All time";
  }

  // Reactive display label
  const displayLabel = $derived(getDisplayLabel());

  /**
   * Format a Date object for HTML date input (YYYY-MM-DD).
   */
  function formatDateForInput(date: Date): string {
    return date.toISOString().split("T")[0];
  }

  /**
   * Format a date string for display (e.g., "Jan 15").
   */
  function formatDisplayDate(dateStr: string): string {
    const date = new SvelteDate(dateStr);
    return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }

  /**
   * Apply a preset date range.
   */
  function applyPreset(preset: "today" | "week" | "month" | "all") {
    const today = new SvelteDate();
    today.setHours(0, 0, 0, 0);

    let start: string | null = null;
    let end: string | null = null;

    switch (preset) {
      case "today":
        start = formatDateForInput(today);
        end = formatDateForInput(today);
        break;
      case "week": {
        const weekAgo = new SvelteDate(today);
        weekAgo.setDate(weekAgo.getDate() - 6);
        start = formatDateForInput(weekAgo);
        end = formatDateForInput(today);
        break;
      }
      case "month": {
        const monthAgo = new SvelteDate(today);
        monthAgo.setDate(monthAgo.getDate() - 29);
        start = formatDateForInput(monthAgo);
        end = formatDateForInput(today);
        break;
      }
      case "all":
        start = null;
        end = null;
        break;
    }

    filtersStore.setDateRange(start, end);
    onChange?.(start, end);
    close();
  }

  /**
   * Handle custom date input changes.
   */
  function handleStartChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const start = target.value || null;
    filtersStore.setDateRange(start, filtersStore.dateEnd);
    onChange?.(start, filtersStore.dateEnd);
  }

  function handleEndChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const end = target.value || null;
    filtersStore.setDateRange(filtersStore.dateStart, end);
    onChange?.(filtersStore.dateStart, end);
  }

  /**
   * Clear the date filter.
   */
  function clearFilter() {
    filtersStore.setDateRange(null, null);
    onChange?.(null, null);
    close();
  }

  function toggle() {
    isOpen = !isOpen;
  }

  function close() {
    isOpen = false;
    buttonRef?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      close();
    }
  }

  // Track document click for outside clicks
  import { onMount, onDestroy } from "svelte";

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
  });
</script>

<div class="date-picker" bind:this={dropdownRef}>
  <button
    bind:this={buttonRef}
    type="button"
    class="picker-button"
    class:has-filter={filtersStore.dateStart || filtersStore.dateEnd}
    aria-haspopup="dialog"
    aria-expanded={isOpen}
    aria-label="Filter by date range"
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <svg
      class="calendar-icon"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
      <line x1="16" y1="2" x2="16" y2="6"></line>
      <line x1="8" y1="2" x2="8" y2="6"></line>
      <line x1="3" y1="10" x2="21" y2="10"></line>
    </svg>
    <span class="button-label">{displayLabel}</span>
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
    <div class="picker-dropdown" role="dialog" aria-label="Date range picker">
      <!-- Quick presets -->
      <div class="presets">
        <button type="button" class="preset-button" onclick={() => applyPreset("today")}>
          Today
        </button>
        <button type="button" class="preset-button" onclick={() => applyPreset("week")}>
          Last 7 days
        </button>
        <button type="button" class="preset-button" onclick={() => applyPreset("month")}>
          Last 30 days
        </button>
        <button type="button" class="preset-button" onclick={() => applyPreset("all")}>
          All time
        </button>
      </div>

      <div class="divider"></div>

      <!-- Custom date inputs -->
      <div class="custom-range">
        <label class="date-field">
          <span class="date-label">From</span>
          <input
            type="date"
            class="date-input"
            value={startDate}
            max={endDate || undefined}
            oninput={handleStartChange}
          />
        </label>
        <label class="date-field">
          <span class="date-label">To</span>
          <input
            type="date"
            class="date-input"
            value={endDate}
            min={startDate || undefined}
            oninput={handleEndChange}
          />
        </label>
      </div>

      <!-- Clear button -->
      {#if filtersStore.dateStart || filtersStore.dateEnd}
        <div class="divider"></div>
        <button type="button" class="clear-button" onclick={clearFilter}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18M6 6l12 12"></path>
          </svg>
          Clear filter
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .date-picker {
    position: relative;
    display: inline-block;
  }

  .picker-button {
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

  .picker-button:hover {
    background-color: var(--color-border-light);
    border-color: var(--color-accent);
  }

  .picker-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .picker-button.has-filter {
    border-color: var(--color-accent);
  }

  .calendar-icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
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

  .picker-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    min-width: 220px;
    margin-top: 4px;
    padding: 0.5rem;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .presets {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .preset-button {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .preset-button:hover {
    background-color: var(--color-bg-tertiary);
  }

  .preset-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .divider {
    height: 1px;
    margin: 0.5rem 0;
    background-color: var(--color-border);
  }

  .custom-range {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0 0.25rem;
  }

  .date-field {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .date-label {
    min-width: 2.5rem;
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .date-input {
    flex: 1;
    padding: 0.375rem 0.5rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-primary);
    font-size: 0.8125rem;
  }

  .date-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .clear-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-error, #ef4444);
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .clear-button:hover {
    background-color: rgba(239, 68, 68, 0.1);
  }

  .clear-button:focus-visible {
    outline: 2px solid var(--color-error, #ef4444);
    outline-offset: -2px;
  }

  .clear-button svg {
    width: 0.875rem;
    height: 0.875rem;
  }
</style>
