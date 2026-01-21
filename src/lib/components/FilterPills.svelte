<script lang="ts">
  /**
   * FilterPills component showing active filters as removable badges.
   *
   * Features:
   * - Pills appear when filters are active
   * - Each pill shows filter type and value
   * - X button on each pill to remove that filter
   * - "Clear all" link when multiple filters active
   */
  import { filtersStore } from "$lib/stores";

  interface Props {
    /** Handler called when filters are cleared */
    onFilterChange?: () => void;
  }

  let { onFilterChange }: Props = $props();

  // Check if any filters are active
  const hasFilters = $derived(filtersStore.hasActiveFilters);

  // Count active filters
  const filterCount = $derived(() => {
    let count = 0;
    if (filtersStore.projectFilter) count++;
    if (filtersStore.dateStart || filtersStore.dateEnd) count++;
    return count;
  });

  // Format date range for display
  function formatDateRange(): string {
    const start = filtersStore.dateStart;
    const end = filtersStore.dateEnd;

    if (start && end) {
      if (start === end) {
        return formatDate(start);
      }
      return `${formatDate(start)} - ${formatDate(end)}`;
    }
    if (start) {
      return `From ${formatDate(start)}`;
    }
    if (end) {
      return `Until ${formatDate(end)}`;
    }
    return "";
  }

  function formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    } catch {
      return dateStr;
    }
  }

  function clearProjectFilter() {
    filtersStore.setProject(null);
    onFilterChange?.();
  }

  function clearDateFilter() {
    filtersStore.setDateRange(null, null);
    onFilterChange?.();
  }

  function clearAllFilters() {
    filtersStore.clearAll();
    onFilterChange?.();
  }
</script>

{#if hasFilters}
  <div class="filter-pills" role="group" aria-label="Active filters">
    <!-- Project filter pill -->
    {#if filtersStore.projectFilter}
      <span class="pill">
        <span class="pill-label">Project:</span>
        <span class="pill-value">{filtersStore.projectFilter}</span>
        <button
          type="button"
          class="pill-remove"
          aria-label="Remove project filter"
          onclick={clearProjectFilter}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18M6 6l12 12"></path>
          </svg>
        </button>
      </span>
    {/if}

    <!-- Date filter pill -->
    {#if filtersStore.dateStart || filtersStore.dateEnd}
      <span class="pill">
        <span class="pill-label">Date:</span>
        <span class="pill-value">{formatDateRange()}</span>
        <button
          type="button"
          class="pill-remove"
          aria-label="Remove date filter"
          onclick={clearDateFilter}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6 6 18M6 6l12 12"></path>
          </svg>
        </button>
      </span>
    {/if}

    <!-- Clear all link when multiple filters -->
    {#if filterCount() > 1}
      <button type="button" class="clear-all" onclick={clearAllFilters}> Clear all filters </button>
    {/if}
  </div>
{/if}

<style>
  .filter-pills {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.375rem 0.25rem 0.5rem;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 16px;
    font-size: 0.75rem;
  }

  .pill-label {
    color: var(--color-text-muted);
  }

  .pill-value {
    color: var(--color-text-primary);
    font-weight: 500;
    max-width: 150px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pill-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    padding: 0;
    margin-left: 0.125rem;
    background: transparent;
    border: none;
    border-radius: 50%;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      background-color 0.1s ease,
      color 0.1s ease;
  }

  .pill-remove:hover {
    background-color: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .pill-remove:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .pill-remove svg {
    width: 0.625rem;
    height: 0.625rem;
  }

  .clear-all {
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: none;
    color: var(--color-accent);
    font-size: 0.75rem;
    cursor: pointer;
    transition: color 0.1s ease;
  }

  .clear-all:hover {
    color: var(--color-text-primary);
    text-decoration: underline;
  }

  .clear-all:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }
</style>
