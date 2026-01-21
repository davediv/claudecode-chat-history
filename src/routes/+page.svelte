<script lang="ts">
  /**
   * Main application page.
   *
   * Layout:
   * - Header bar with search and filters
   * - Two-column split: sidebar (conversation list) and detail pane
   */
  import Header from "$lib/components/Header.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPane from "$lib/components/DetailPane.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";

  // Search state
  let searchQuery = $state("");

  // Conversation list state
  let conversations = $state<
    Array<{
      id: string;
      projectName: string;
      preview: string;
      lastTime: string;
      messageCount: number;
    }>
  >([]);
  let selectedConversationId = $state<string | null>(null);
  let isLoadingList = $state(false);
  let isLoadingDetail = $state(false);

  function handleSearch(query: string) {
    searchQuery = query;
    // TODO: Implement search filtering
  }

  function handleSelectConversation(id: string) {
    selectedConversationId = id;
    // TODO: Load conversation detail
  }
</script>

<div class="app-layout">
  <Header {searchQuery} onSearch={handleSearch} />

  <div class="main-content">
    <Sidebar
      {conversations}
      selectedId={selectedConversationId}
      onSelect={handleSelectConversation}
      isLoading={isLoadingList}
    />

    <DetailPane hasSelection={selectedConversationId !== null} isLoading={isLoadingDetail}>
      <!-- Conversation content will be rendered here -->
    </DetailPane>
  </div>

  <!-- Toast notifications -->
  <ToastContainer />
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
