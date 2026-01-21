<script lang="ts">
  /**
   * Sidebar component wrapping the conversation list.
   *
   * Features:
   * - Fixed width sidebar container
   * - Contains ConversationList with virtual scrolling
   * - Responsive: hidden on narrow screens
   */
  import ConversationList from "./ConversationList.svelte";

  interface ConversationItem {
    id: string;
    projectName: string;
    preview: string;
    lastTime: string;
    messageCount: number;
    bookmarked: boolean;
  }

  interface Props {
    /** List of conversations to display */
    conversations?: ConversationItem[];
    /** Currently selected conversation ID */
    selectedId?: string | null;
    /** Handler for conversation selection */
    onSelect?: (id: string) => void;
    /** Handler for bookmark toggle */
    onToggleBookmark?: (id: string) => void;
    /** Whether data is loading */
    isLoading?: boolean;
    /** Reference to the conversation list element (bindable) */
    listRef?: HTMLElement;
  }

  let {
    conversations = [],
    selectedId = null,
    onSelect,
    onToggleBookmark,
    isLoading = false,
    listRef = $bindable(),
  }: Props = $props();
</script>

<aside class="sidebar" aria-label="Conversation list">
  <ConversationList
    {conversations}
    {selectedId}
    {onSelect}
    {onToggleBookmark}
    {isLoading}
    bind:listRef
  />
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background-color: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
  }

  /* Responsive: hide sidebar on narrow screens */
  @media (max-width: 640px) {
    .sidebar {
      position: absolute;
      left: 0;
      top: var(--header-height);
      bottom: 0;
      z-index: 10;
      transform: translateX(-100%);
      transition: transform 0.2s ease;
    }
  }
</style>
