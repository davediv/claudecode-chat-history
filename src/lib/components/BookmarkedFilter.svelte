<script lang="ts">
  /**
   * Bookmarked filter dropdown component.
   *
   * Features:
   * - Filter by bookmarked status (All, Bookmarked, Not Bookmarked)
   * - Integrates with filters store
   */
  import FilterDropdown from "./FilterDropdown.svelte";
  import { filtersStore } from "$lib/stores";

  interface Props {
    /** Handler for filter changes */
    onChange?: () => void;
  }

  let { onChange }: Props = $props();

  // Options for the bookmarked filter
  const options = [
    { value: "true", label: "Bookmarked" },
    { value: "false", label: "Not Bookmarked" },
  ];

  // Get current selected value from store
  const selected = $derived(() => {
    if (filtersStore.bookmarkedFilter === null) return "";
    return filtersStore.bookmarkedFilter ? "true" : "false";
  });

  function handleChange(value: string) {
    if (value === "") {
      filtersStore.setBookmarked(null);
    } else {
      filtersStore.setBookmarked(value === "true");
    }
    onChange?.();
  }
</script>

<FilterDropdown
  {options}
  selected={selected()}
  placeholder="Bookmarks"
  allLabel="All"
  onChange={handleChange}
  ariaLabel="Filter by bookmark status"
/>
