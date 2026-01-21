<script lang="ts">
  /**
   * Conversation card component for displaying a single conversation summary.
   *
   * Features:
   * - Project name, relative date, preview text
   * - Preview truncated to 100 chars with ellipsis
   * - Hover and selected states
   */

  interface Props {
    /** Unique conversation ID */
    id: string;
    /** Project name to display */
    projectName?: string;
    /** Preview text (first message excerpt) */
    preview?: string;
    /** Last activity time (ISO 8601) */
    lastTime?: string;
    /** Number of messages in conversation */
    messageCount?: number;
    /** Whether this conversation is bookmarked */
    bookmarked?: boolean;
    /** Whether this card is currently selected */
    isSelected?: boolean;
    /** Handler for card selection */
    onSelect?: (id: string) => void;
    /** Handler for bookmark toggle */
    onToggleBookmark?: (id: string) => void;
  }

  let {
    id,
    projectName = "Unknown project",
    preview = "No preview available",
    lastTime,
    messageCount = 0,
    bookmarked = false,
    isSelected = false,
    onSelect,
    onToggleBookmark,
  }: Props = $props();

  // Display values with fallbacks for missing data
  const displayProjectName = $derived(projectName || "Unknown project");
  const displayPreview = $derived(preview || "No preview available");

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

  /**
   * Truncate preview text to 100 characters with ellipsis
   */
  function truncatePreview(text: string, maxLength: number = 100): string {
    if (text.length <= maxLength) {
      return text;
    }
    return text.slice(0, maxLength).trimEnd() + "…";
  }

  function handleClick() {
    onSelect?.(id);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect?.(id);
    }
  }

  function handleBookmarkClick(event: MouseEvent) {
    event.stopPropagation();
    onToggleBookmark?.(id);
  }

  function handleBookmarkKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      onToggleBookmark?.(id);
    }
  }
</script>

<div
  class="conversation-card"
  class:selected={isSelected}
  role="option"
  aria-selected={isSelected}
  tabindex="0"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <div class="card-header">
    <span class="project-name" title={displayProjectName}>{displayProjectName}</span>
    <span class="timestamp">{lastTime ? formatRelativeDate(lastTime) : "Unknown date"}</span>
  </div>

  <p class="preview" class:placeholder={!preview} title={displayPreview}>
    {truncatePreview(displayPreview)}
  </p>

  <div class="card-footer">
    <span class="message-count">
      <svg
        class="message-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
      </svg>
      {messageCount}
    </span>
    <button
      class="bookmark-button"
      class:bookmarked
      onclick={handleBookmarkClick}
      onkeydown={handleBookmarkKeydown}
      aria-label={bookmarked ? "Remove bookmark" : "Bookmark conversation"}
      aria-pressed={bookmarked}
      title={bookmarked ? "Remove bookmark" : "Bookmark conversation"}
    >
      <svg
        class="bookmark-icon"
        viewBox="0 0 24 24"
        fill={bookmarked ? "currentColor" : "none"}
        stroke="currentColor"
        stroke-width="2"
      >
        <path
          d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
        ></path>
      </svg>
    </button>
  </div>
</div>

<style>
  .conversation-card {
    padding: 0.75rem 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s ease;
    outline: none;
  }

  .conversation-card:hover {
    background-color: var(--color-bg-tertiary);
  }

  .conversation-card.selected {
    background-color: var(--color-bg-tertiary);
    border-left: 3px solid var(--color-accent);
    padding-left: calc(1rem - 3px);
  }

  .conversation-card:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
    gap: 0.5rem;
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

  .timestamp {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .preview {
    margin: 0 0 0.375rem 0;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    line-height: 1.4;
    /* Two-line clamp fallback */
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .preview.placeholder {
    color: var(--color-text-muted);
    font-style: italic;
  }

  .card-footer {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .message-count {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  .message-icon {
    width: 0.75rem;
    height: 0.75rem;
  }

  .bookmark-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    padding: 0;
    margin-left: auto;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      color 0.15s ease,
      background-color 0.15s ease;
  }

  .bookmark-button:hover {
    color: var(--color-text-secondary);
    background-color: var(--color-bg-tertiary);
  }

  .bookmark-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .bookmark-button.bookmarked {
    color: var(--color-accent);
  }

  .bookmark-button.bookmarked:hover {
    color: var(--color-accent);
    background-color: var(--color-bg-tertiary);
  }

  .bookmark-icon {
    width: 0.875rem;
    height: 0.875rem;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .bookmark-button {
      transition: none;
    }
  }
</style>
