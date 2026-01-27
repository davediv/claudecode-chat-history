<script lang="ts">
  /**
   * Main application page.
   *
   * Layout:
   * - Header bar with search and filters
   * - Two-column split: sidebar (conversation list) and detail pane
   * - Error boundaries for graceful error handling
   *
   * Keyboard navigation:
   * - `/` focuses search (handled by SearchInput)
   * - `j/k` navigates conversation list (handled by ConversationList)
   * - `Escape` clears selection and closes modals
   * - `Enter` activates focused element
   */
  import { onMount, onDestroy } from "svelte";
  import Header from "$lib/components/Header.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPane from "$lib/components/DetailPane.svelte";
  import ConversationDetail from "$lib/components/ConversationDetail.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import AnalyticsModal from "$lib/components/AnalyticsModal.svelte";
  import { ErrorBoundary, FilterPills } from "$lib/components";
  import { conversationsStore, tagsStore, uiStore } from "$lib/stores";
  import { setTags as setTagsService } from "$lib/services/tauri";

  // Local loading state for detail pane (store handles list loading)
  let isLoadingDetail = $state(false);

  function handleSearch() {
    // Search is now handled by the searchStore in Header component
    // This callback can be used for additional page-level side effects
  }

  function handleFilterChange() {
    // Called when filters are cleared from FilterPills
    // Trigger a reload of conversations with new filters
    conversationsStore.load();
  }

  async function handleSelectConversation(id: string) {
    isLoadingDetail = true;
    try {
      await conversationsStore.select(id);
    } finally {
      isLoadingDetail = false;
    }
  }

  function handleBack() {
    conversationsStore.clearSelection();
  }

  /**
   * Handle bookmark toggle for a conversation.
   * Updates both the list and detail views via the store.
   */
  async function handleToggleBookmark(id: string) {
    await conversationsStore.toggleBookmark(id);
  }

  /**
   * Handle tag changes for a conversation.
   */
  async function handleTagsChange(id: string, tags: string[]) {
    try {
      const normalizedTags = await setTagsService(id, tags);

      // Update selected conversation with normalized tags
      if (
        conversationsStore.selectedConversation &&
        conversationsStore.selectedConversation.id === id
      ) {
        conversationsStore.setSelectedConversation({
          ...conversationsStore.selectedConversation,
          tags: normalizedTags,
        });
      }

      // Refresh the tags list to update autocomplete
      await tagsStore.refresh();
    } catch (error) {
      console.error("Failed to update tags:", error);
    }
  }

  // Reference to conversation list for keyboard focus
  let conversationListRef: HTMLElement | undefined = $state();

  /**
   * Global keyboard handler for page-level shortcuts.
   * Individual components handle their own shortcuts (/, j/k, etc.)
   * This handles Escape at page level for clearing selection.
   */
  function handleGlobalKeydown(event: KeyboardEvent) {
    // Ignore if typing in an input, textarea, or contenteditable
    const target = event.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) {
      return;
    }

    switch (event.key) {
      case "Escape":
        // Close analytics modal first if open
        if (uiStore.analyticsModalOpen) {
          event.preventDefault();
          uiStore.closeAnalyticsModal();
        }
        // If conversation is selected, clear selection and focus list
        else if (conversationsStore.selectedId) {
          event.preventDefault();
          handleBack();
          // Focus the conversation list for keyboard navigation
          conversationListRef?.focus();
        }
        break;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleGlobalKeydown);
    // Load tags for autocomplete
    tagsStore.load();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleGlobalKeydown);
  });
</script>

<div class="app-layout">
  <Header onSearch={handleSearch} />
  <FilterPills onFilterChange={handleFilterChange} />

  <div id="main-content" class="main-content">
    <ErrorBoundary title="Sidebar error" description="Failed to load the conversation list.">
      <Sidebar
        conversations={conversationsStore.conversations}
        selectedId={conversationsStore.selectedId}
        onSelect={handleSelectConversation}
        onToggleBookmark={handleToggleBookmark}
        isLoading={conversationsStore.loading}
        bind:listRef={conversationListRef}
      />
    </ErrorBoundary>

    <ErrorBoundary title="Content error" description="Failed to load the conversation details.">
      <DetailPane hasSelection={conversationsStore.selectedId !== null} isLoading={isLoadingDetail}>
        {#if conversationsStore.selectedConversation}
          <ConversationDetail
            conversation={conversationsStore.selectedConversation}
            onBack={handleBack}
            onToggleBookmark={handleToggleBookmark}
            onTagsChange={handleTagsChange}
            allTags={tagsStore.allTags}
          />
        {/if}
      </DetailPane>
    </ErrorBoundary>
  </div>

  <!-- Toast notifications -->
  <ToastContainer />

  <!-- Analytics modal -->
  {#if uiStore.analyticsModalOpen}
    <AnalyticsModal conversations={conversationsStore.conversations} />
  {/if}
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background-color: var(--color-bg-primary);
    color: var(--color-text-primary);
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  /* Ensure no horizontal scroll at 800px */
  @media (min-width: 800px) {
    .main-content {
      overflow-x: hidden;
    }
  }
</style>
