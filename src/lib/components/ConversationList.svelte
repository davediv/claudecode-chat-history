<script lang="ts">
  /**
   * Conversation list component with virtual scrolling.
   *
   * Features:
   * - Virtual scrolling for 10,000+ conversations
   * - Loading skeleton during initial load
   * - Keyboard navigation: j/k to move, Enter to open
   * - Empty state when no conversations
   */
  import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
  import ConversationCard from "./ConversationCard.svelte";
  import ConversationListSkeleton from "./ConversationListSkeleton.svelte";

  interface ConversationItem {
    id: string;
    projectName: string;
    preview: string;
    lastTime: string;
    messageCount: number;
  }

  interface Props {
    /** List of conversations to display */
    conversations: ConversationItem[];
    /** Currently selected conversation ID */
    selectedId?: string | null;
    /** Handler for conversation selection */
    onSelect?: (id: string) => void;
    /** Whether data is loading */
    isLoading?: boolean;
    /** Reference to the list element (bindable) */
    listRef?: HTMLElement;
  }

  let {
    conversations,
    selectedId = null,
    onSelect,
    isLoading = false,
    listRef = $bindable(),
  }: Props = $props();

  // Reference to virtual list for programmatic scrolling
  let virtualListRef: SvelteVirtualList<ConversationItem> | undefined = $state();

  // Currently focused index for keyboard navigation
  let focusedIndex = $state(0);

  // Update focused index when selection changes externally
  $effect(() => {
    if (selectedId) {
      const idx = conversations.findIndex((c) => c.id === selectedId);
      if (idx !== -1) {
        focusedIndex = idx;
      }
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (conversations.length === 0) return;

    switch (event.key) {
      case "j":
      case "ArrowDown": {
        event.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, conversations.length - 1);
        scrollToIndex(focusedIndex);
        break;
      }
      case "k":
      case "ArrowUp": {
        event.preventDefault();
        focusedIndex = Math.max(focusedIndex - 1, 0);
        scrollToIndex(focusedIndex);
        break;
      }
      case "Enter": {
        event.preventDefault();
        const conversation = conversations[focusedIndex];
        if (conversation) {
          onSelect?.(conversation.id);
        }
        break;
      }
      case "Home": {
        event.preventDefault();
        focusedIndex = 0;
        scrollToIndex(0);
        break;
      }
      case "End": {
        event.preventDefault();
        focusedIndex = conversations.length - 1;
        scrollToIndex(focusedIndex);
        break;
      }
    }
  }

  function scrollToIndex(index: number) {
    virtualListRef?.scroll({
      index,
      smoothScroll: true,
      align: "nearest",
      shouldThrowOnBounds: false,
    });
  }

  function handleSelect(id: string) {
    const idx = conversations.findIndex((c) => c.id === id);
    if (idx !== -1) {
      focusedIndex = idx;
    }
    onSelect?.(id);
  }
</script>

<div
  bind:this={listRef}
  class="conversation-list"
  role="listbox"
  aria-label="Conversations"
  tabindex="0"
  onkeydown={handleKeydown}
>
  {#if isLoading}
    <ConversationListSkeleton count={8} />
  {:else if conversations.length === 0}
    <div class="empty-state">
      <svg
        class="empty-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path
          d="M20 13V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7m16 0v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-5m16 0h-2.586a1 1 0 0 0-.707.293l-2.414 2.414a1 1 0 0 1-.707.293h-3.172a1 1 0 0 1-.707-.293l-2.414-2.414A1 1 0 0 0 6.586 13H4"
        ></path>
      </svg>
      <p class="empty-title">No Claude Code history found</p>
      <p class="empty-description">Start a conversation in Claude Code to see it here.</p>
      <a
        href="https://docs.anthropic.com/en/docs/claude-code"
        target="_blank"
        rel="noopener noreferrer"
        class="empty-link"
      >
        Learn about Claude Code
        <svg
          class="link-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
          <polyline points="15 3 21 3 21 9"></polyline>
          <line x1="10" y1="14" x2="21" y2="3"></line>
        </svg>
      </a>
    </div>
  {:else}
    <SvelteVirtualList
      bind:this={virtualListRef}
      items={conversations}
      defaultEstimatedItemHeight={72}
      bufferSize={10}
      containerClass="virtual-list-container"
      viewportClass="virtual-list-viewport"
    >
      {#snippet renderItem(item, index)}
        <ConversationCard
          id={item.id}
          projectName={item.projectName}
          preview={item.preview}
          lastTime={item.lastTime}
          messageCount={item.messageCount}
          isSelected={selectedId === item.id || focusedIndex === index}
          onSelect={handleSelect}
        />
      {/snippet}
    </SvelteVirtualList>
  {/if}
</div>

<style>
  .conversation-list {
    height: 100%;
    width: 100%;
    overflow: hidden;
    outline: none;
    background-color: var(--color-bg-secondary);
  }

  .conversation-list:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  /* Virtual list container styles */
  .conversation-list :global(.virtual-list-container) {
    height: 100%;
  }

  .conversation-list :global(.virtual-list-viewport) {
    height: 100%;
    overflow-y: auto;
  }

  /* Empty state styles */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .empty-icon {
    width: 3rem;
    height: 3rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .empty-title {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .empty-description {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .empty-link {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    color: var(--color-accent);
    text-decoration: none;
    border: 1px solid var(--color-accent);
    border-radius: 6px;
    transition: background-color 0.15s ease;
  }

  .empty-link:hover {
    background-color: rgba(59, 130, 246, 0.1);
  }

  .link-icon {
    width: 0.875rem;
    height: 0.875rem;
  }
</style>
