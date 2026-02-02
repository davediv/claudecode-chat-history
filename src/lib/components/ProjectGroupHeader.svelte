<script lang="ts">
  /**
   * Project group header component for grouped conversation view.
   *
   * Features:
   * - Chevron icon that rotates on expand/collapse
   * - Project name with conversation count badge
   * - Last activity timestamp
   * - Click/keyboard to toggle expand/collapse
   */

  interface Props {
    /** Project name to display */
    projectName: string;
    /** Number of conversations in this project */
    count: number;
    /** Most recent activity timestamp (ISO 8601) */
    lastActivity?: string;
    /** Whether this project section is expanded */
    isExpanded: boolean;
    /** Whether this header is focused for keyboard navigation */
    isFocused?: boolean;
    /** Whether this project contains the currently selected conversation */
    hasSelectedChild?: boolean;
    /** Handler for toggle action */
    onToggle: () => void;
  }

  let {
    projectName,
    count,
    lastActivity,
    isExpanded,
    isFocused = false,
    hasSelectedChild = false,
    onToggle,
  }: Props = $props();

  /**
   * Format a date as relative ("2 hours ago") if < 7 days,
   * otherwise as "Jan 15, 2025"
   */
  function formatRelativeDate(isoString: string): string {
    try {
      const date = new Date(isoString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffSeconds = Math.floor(diffMs / 1000);
      const diffMinutes = Math.floor(diffSeconds / 60);
      const diffHours = Math.floor(diffMinutes / 60);
      const diffDays = Math.floor(diffHours / 24);

      if (diffSeconds < 60) {
        return "just now";
      } else if (diffMinutes < 60) {
        return `${diffMinutes} ${diffMinutes === 1 ? "minute" : "minutes"} ago`;
      } else if (diffHours < 24) {
        return `${diffHours} ${diffHours === 1 ? "hour" : "hours"} ago`;
      } else if (diffDays === 1) {
        return "yesterday";
      } else if (diffDays < 7) {
        return `${diffDays} days ago`;
      } else {
        return date.toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
          year: "numeric",
        });
      }
    } catch {
      return "";
    }
  }

  function handleClick() {
    onToggle();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      onToggle();
    } else if (event.key === "ArrowLeft" && isExpanded) {
      event.preventDefault();
      event.stopPropagation();
      onToggle();
    } else if (event.key === "ArrowRight" && !isExpanded) {
      event.preventDefault();
      event.stopPropagation();
      onToggle();
    }
  }
</script>

<div
  class="project-header"
  class:focused={isFocused}
  class:has-selected-child={hasSelectedChild}
  role="button"
  tabindex="0"
  aria-expanded={isExpanded}
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <svg
    class="chevron"
    class:expanded={isExpanded}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    aria-hidden="true"
  >
    <polyline points="9 18 15 12 9 6"></polyline>
  </svg>

  <span class="project-name" title={projectName}>{projectName}</span>

  <span class="count-badge" title="{count} conversation{count !== 1 ? 's' : ''}">
    {count}
  </span>

  {#if lastActivity}
    <span class="last-activity">{formatRelativeDate(lastActivity)}</span>
  {/if}
</div>

<style>
  .project-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background-color: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s ease;
    outline: none;
  }

  .project-header:hover {
    background-color: color-mix(in srgb, var(--color-bg-tertiary) 80%, var(--color-accent) 20%);
  }

  .project-header.focused {
    background-color: color-mix(in srgb, var(--color-bg-tertiary) 70%, var(--color-accent) 30%);
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .project-header.has-selected-child {
    border-left: 3px solid var(--color-accent);
    padding-left: calc(1rem - 3px);
  }

  .project-header:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .chevron {
    width: 1rem;
    height: 1rem;
    color: var(--color-text-muted);
    transition: transform 0.15s ease;
    flex-shrink: 0;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .project-name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .count-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.5rem;
    height: 1.25rem;
    padding: 0 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-secondary);
    border-radius: 0.625rem;
    flex-shrink: 0;
  }

  .last-activity {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .project-header,
    .chevron {
      transition: none;
    }
  }
</style>
