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
  import ConversationDetail from "$lib/components/ConversationDetail.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import type { Conversation, Message, ContentBlock } from "$lib/types";

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
  let selectedConversation = $state<Conversation | null>(null);
  let isLoadingList = $state(false);
  let isLoadingDetail = $state(false);

  // Development: Generate mock data for testing virtual scrolling
  // Remove this in production when connecting to actual Tauri API
  $effect(() => {
    if (import.meta.env.DEV) {
      // Generate 1500 mock conversations for testing
      conversations = Array.from({ length: 1500 }, (_, i) => ({
        id: `conv-${i}`,
        projectName: `project-${Math.floor(i / 50)}`,
        preview: `This is a preview message for conversation ${i}. It contains some sample text that demonstrates truncation when the content exceeds 100 characters.`,
        lastTime: new Date(Date.now() - i * 60 * 60 * 1000).toISOString(),
        messageCount: Math.floor(Math.random() * 50) + 1,
      }));
    }
  });

  /**
   * Generate mock messages for a selected conversation (development only)
   */
  function generateMockConversation(id: string): Conversation {
    const convIndex = parseInt(id.replace("conv-", ""), 10);
    const messageCount = Math.floor(Math.random() * 30) + 5;
    const baseTime = Date.now() - convIndex * 60 * 60 * 1000;

    const messages: Message[] = Array.from({ length: messageCount }, (_, i) => {
      const role: Message["role"] = i % 2 === 0 ? "user" : "assistant";
      const content: ContentBlock[] = [
        {
          type: "text",
          content:
            role === "user"
              ? `This is user message ${i + 1}. How can I implement a feature that does something interesting?`
              : `Here's my response to your question. Let me explain the approach step by step.\n\nFirst, you would want to consider the architecture and how the components interact with each other.\n\nSecond, implement the core logic with proper error handling.\n\nThird, add tests to ensure everything works correctly.`,
        },
      ];

      return {
        id: `msg-${id}-${i}`,
        role,
        content,
        timestamp: new Date(baseTime + i * 5 * 60 * 1000).toISOString(),
        tokenCount: {
          input: Math.floor(Math.random() * 100),
          output: Math.floor(Math.random() * 500),
        },
      };
    });

    return {
      id,
      projectPath: `/Users/dev/projects/project-${Math.floor(convIndex / 50)}`,
      projectName: `project-${Math.floor(convIndex / 50)}`,
      startTime: new Date(baseTime).toISOString(),
      lastTime: new Date(baseTime + (messageCount - 1) * 5 * 60 * 1000).toISOString(),
      messages,
      totalTokens: { input: 500, output: 2000 },
    };
  }

  function handleSearch(query: string) {
    searchQuery = query;
    // TODO: Implement search filtering
  }

  function handleSelectConversation(id: string) {
    selectedConversationId = id;
    isLoadingDetail = true;

    // Simulate loading delay in development
    if (import.meta.env.DEV) {
      setTimeout(() => {
        selectedConversation = generateMockConversation(id);
        isLoadingDetail = false;
      }, 200);
    }
  }

  function handleBack() {
    selectedConversationId = null;
    selectedConversation = null;
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
      {#if selectedConversation}
        <ConversationDetail conversation={selectedConversation} onBack={handleBack} />
      {/if}
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
