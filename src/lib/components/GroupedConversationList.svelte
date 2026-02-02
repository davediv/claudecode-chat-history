<script lang="ts">
  /**
   * Grouped conversation list component with virtual scrolling.
   *
   * Features:
   * - Groups conversations by project with collapsible sections
   * - Virtual scrolling for performance with many conversations
   * - Keyboard navigation: j/k to move, Enter to open/toggle, arrows to expand/collapse
   * - Flattened list pattern for heterogeneous items (headers + cards)
   */
  import SvelteVirtualList from "@humanspeak/svelte-virtual-list";
  import ConversationCard from "./ConversationCard.svelte";
  import ProjectGroupHeader from "./ProjectGroupHeader.svelte";
  import ConversationListSkeleton from "./ConversationListSkeleton.svelte";
  import { SvelteMap } from "svelte/reactivity";
  import type { GroupedListItem, ConversationSummary } from "$lib/types";
  import { groupedViewStore } from "$lib/stores";

  interface Props {
    /** List of conversations to display */
    conversations: ConversationSummary[];
    /** Currently selected conversation ID */
    selectedId?: string | null;
    /** Handler for conversation selection */
    onSelect?: (id: string) => void;
    /** Handler for bookmark toggle */
    onToggleBookmark?: (id: string) => void;
    /** Whether data is loading */
    isLoading?: boolean;
    /** Reference to the list element (bindable) */
    listRef?: HTMLElement;
  }

  let {
    conversations,
    selectedId = null,
    onSelect,
    onToggleBookmark,
    isLoading = false,
    listRef = $bindable(),
  }: Props = $props();

  // Reference to virtual list for programmatic scrolling
  let virtualListRef: SvelteVirtualList<GroupedListItem> | undefined = $state();

  // Currently focused index for keyboard navigation
  let focusedIndex = $state(0);

  /**
   * Group conversations by project and build a flattened list.
   * Each project gets a header, followed by its conversations if expanded.
   */
  const groupedItems = $derived.by((): GroupedListItem[] => {
    if (conversations.length === 0) return [];

    // Group by project name using SvelteMap for reactivity
    const projectMap = new SvelteMap<
      string,
      { conversations: ConversationSummary[]; lastActivity: string }
    >();

    for (const conv of conversations) {
      const existing = projectMap.get(conv.projectName);
      if (existing) {
        existing.conversations.push(conv);
        // Update lastActivity if this conversation is more recent
        if (conv.lastTime > existing.lastActivity) {
          existing.lastActivity = conv.lastTime;
        }
      } else {
        projectMap.set(conv.projectName, {
          conversations: [conv],
          lastActivity: conv.lastTime,
        });
      }
    }

    // Sort projects by most recent activity
    const sortedProjects = Array.from(projectMap.entries()).sort(([, a], [, b]) =>
      b.lastActivity.localeCompare(a.lastActivity)
    );

    // Build flattened list
    const items: GroupedListItem[] = [];

    for (const [projectName, { conversations: projConvs, lastActivity }] of sortedProjects) {
      const isExpanded = groupedViewStore.isProjectExpanded(projectName);

      // Add project header
      items.push({
        type: "project-header",
        projectName,
        conversationCount: projConvs.length,
        lastActivity,
        isExpanded,
      });

      // Add conversations if expanded
      if (isExpanded) {
        // Sort conversations within project by most recent first
        const sortedConvs = [...projConvs].sort((a, b) => b.lastTime.localeCompare(a.lastTime));
        for (const conv of sortedConvs) {
          items.push({
            type: "conversation",
            conversation: conv,
          });
        }
      }
    }

    return items;
  });

  // Get project name of the selected conversation
  const selectedProjectName = $derived.by(() => {
    if (!selectedId) return null;
    const conv = conversations.find((c) => c.id === selectedId);
    return conv?.projectName ?? null;
  });

  // Update focused index when selection changes externally
  $effect(() => {
    if (selectedId) {
      const idx = groupedItems.findIndex(
        (item) => item.type === "conversation" && item.conversation.id === selectedId
      );
      if (idx !== -1) {
        focusedIndex = idx;
      }
    }
  });

  function handleKeydown(event: KeyboardEvent) {
    if (groupedItems.length === 0) return;

    const currentItem = groupedItems[focusedIndex];

    switch (event.key) {
      case "j":
      case "ArrowDown": {
        event.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, groupedItems.length - 1);
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
        if (currentItem?.type === "project-header") {
          groupedViewStore.toggleProject(currentItem.projectName);
        } else if (currentItem?.type === "conversation") {
          onSelect?.(currentItem.conversation.id);
        }
        break;
      }
      case "ArrowLeft": {
        event.preventDefault();
        if (currentItem?.type === "project-header" && currentItem.isExpanded) {
          // Collapse expanded header
          groupedViewStore.collapseProject(currentItem.projectName);
        } else if (currentItem?.type === "conversation") {
          // Jump to parent header
          const projectName = currentItem.conversation.projectName;
          const headerIdx = groupedItems.findIndex(
            (item) => item.type === "project-header" && item.projectName === projectName
          );
          if (headerIdx !== -1) {
            focusedIndex = headerIdx;
            scrollToIndex(focusedIndex);
          }
        }
        break;
      }
      case "ArrowRight": {
        event.preventDefault();
        if (currentItem?.type === "project-header" && !currentItem.isExpanded) {
          // Expand collapsed header
          groupedViewStore.expandProject(currentItem.projectName);
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
        focusedIndex = groupedItems.length - 1;
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
    const idx = groupedItems.findIndex(
      (item) => item.type === "conversation" && item.conversation.id === id
    );
    if (idx !== -1) {
      focusedIndex = idx;
    }
    onSelect?.(id);
  }

  function handleToggleProject(projectName: string) {
    groupedViewStore.toggleProject(projectName);
  }
</script>

<div
  bind:this={listRef}
  class="grouped-list"
  role="tree"
  aria-label="Conversations grouped by project"
  tabindex="0"
  onkeydown={handleKeydown}
>
  {#if isLoading}
    <ConversationListSkeleton count={8} />
  {:else if groupedItems.length === 0}
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
      items={groupedItems}
      defaultEstimatedItemHeight={48}
      bufferSize={10}
      containerClass="virtual-list-container"
      viewportClass="virtual-list-viewport"
    >
      {#snippet renderItem(item, index)}
        {#if item.type === "project-header"}
          <ProjectGroupHeader
            projectName={item.projectName}
            count={item.conversationCount}
            lastActivity={item.lastActivity}
            isExpanded={item.isExpanded}
            isFocused={focusedIndex === index}
            hasSelectedChild={item.projectName === selectedProjectName}
            onToggle={() => handleToggleProject(item.projectName)}
          />
        {:else}
          <ConversationCard
            id={item.conversation.id}
            hideProjectName={true}
            preview={item.conversation.preview}
            lastTime={item.conversation.lastTime}
            messageCount={item.conversation.messageCount}
            bookmarked={item.conversation.bookmarked}
            subagentCount={item.conversation.subagentCount}
            isSelected={selectedId === item.conversation.id || focusedIndex === index}
            onSelect={handleSelect}
            {onToggleBookmark}
          />
        {/if}
      {/snippet}
    </SvelteVirtualList>
  {/if}
</div>

<style>
  .grouped-list {
    height: 100%;
    width: 100%;
    overflow: hidden;
    outline: none;
    background-color: var(--color-bg-secondary);
  }

  .grouped-list:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  /* Virtual list container styles */
  .grouped-list :global(.virtual-list-container) {
    height: 100%;
  }

  .grouped-list :global(.virtual-list-viewport) {
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
