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
  import { tagsStore, uiStore } from "$lib/stores";
  import { setTags as setTagsService } from "$lib/services/tauri";
  import type { Conversation, Message, ContentBlock } from "$lib/types";

  // Conversation list state
  let conversations = $state<
    Array<{
      id: string;
      projectName: string;
      preview: string;
      lastTime: string;
      messageCount: number;
      bookmarked: boolean;
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
        bookmarked: i % 10 === 0, // Every 10th conversation is bookmarked for testing
      }));
    }
  });

  /**
   * Generate mock messages for a selected conversation (development only)
   */
  function generateMockConversation(id: string): Conversation {
    const convIndex = parseInt(id.replace("conv-", ""), 10);
    const messageCount = Math.floor(Math.random() * 20) + 6;
    const baseTime = Date.now() - convIndex * 60 * 60 * 1000;

    const messages: Message[] = Array.from({ length: messageCount }, (_, i) => {
      const role: Message["role"] = i % 2 === 0 ? "user" : "assistant";

      // Generate varied content blocks for testing
      const content: ContentBlock[] = [];

      if (role === "user") {
        content.push({
          type: "text",
          content: `This is user message ${i + 1}. How can I implement a feature that does something interesting?`,
        });
        // Add code block to some user messages
        if (i % 4 === 0) {
          content.push({
            type: "code",
            language: "typescript",
            content: `function example() {\n  console.log("Hello from user");\n  return true;\n}`,
          });
        }
      } else {
        content.push({
          type: "text",
          content: `Here's my response to your question. Let me explain the approach step by step.`,
        });

        // Add code block to assistant messages
        if (i % 2 === 1) {
          content.push({
            type: "code",
            language: i % 4 === 1 ? "typescript" : i % 4 === 3 ? "python" : "rust",
            content:
              i % 4 === 1
                ? `interface User {\n  id: string;\n  name: string;\n  email: string;\n}\n\nfunction createUser(data: Partial<User>): User {\n  return {\n    id: crypto.randomUUID(),\n    name: data.name || "Unknown",\n    email: data.email || "",\n  };\n}`
                : i % 4 === 3
                  ? `def process_data(items: list) -> dict:\n    """Process a list of items and return stats."""\n    return {\n        "count": len(items),\n        "unique": len(set(items)),\n    }`
                  : `fn main() {\n    let greeting = "Hello, Rust!";\n    println!("{}", greeting);\n}`,
          });
        }

        // Add tool use/result to some messages
        if (i % 6 === 1) {
          content.push({
            type: "tool_use",
            toolName: "Read",
            content: `{\n  "file_path": "/src/lib/components/Example.svelte"\n}`,
          });
        } else if (i % 6 === 3) {
          content.push({
            type: "tool_result",
            toolName: "Bash",
            content: `npm run build\n\n> project@1.0.0 build\n> vite build\n\n✓ 42 modules transformed.\nDone in 1.23s`,
          });
        }

        content.push({
          type: "text",
          content: `That should help you get started with the implementation.`,
        });
      }

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

    // Get bookmark status from conversations list
    const conv = conversations.find((c) => c.id === id);
    const bookmarked = conv?.bookmarked ?? false;

    return {
      id,
      projectPath: `/Users/dev/projects/project-${Math.floor(convIndex / 50)}`,
      projectName: `project-${Math.floor(convIndex / 50)}`,
      startTime: new Date(baseTime).toISOString(),
      lastTime: new Date(baseTime + (messageCount - 1) * 5 * 60 * 1000).toISOString(),
      messages,
      totalTokens: { input: 500, output: 2000 },
      bookmarked,
    };
  }

  function handleSearch() {
    // Search is now handled by the searchStore in Header component
    // This callback can be used for additional page-level side effects
  }

  function handleFilterChange() {
    // Called when filters are cleared from FilterPills
    // In production, this would trigger a reload of conversations with new filters
    // For development, the mock data doesn't need to be refreshed
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

  /**
   * Handle bookmark toggle for a conversation.
   * Updates both the list and detail views.
   */
  function handleToggleBookmark(id: string) {
    // Toggle bookmark in the conversations list
    conversations = conversations.map((c) =>
      c.id === id ? { ...c, bookmarked: !c.bookmarked } : c
    );

    // Update the selected conversation if it matches
    if (selectedConversation && selectedConversation.id === id) {
      selectedConversation = {
        ...selectedConversation,
        bookmarked: !selectedConversation.bookmarked,
      };
    }

    // In production, this would also call the Tauri backend:
    // conversationsStore.toggleBookmark(id);
  }

  /**
   * Handle tag changes for a conversation.
   */
  async function handleTagsChange(id: string, tags: string[]) {
    if (import.meta.env.DEV) {
      // Mock implementation for development - just update local state
      if (selectedConversation && selectedConversation.id === id) {
        selectedConversation = {
          ...selectedConversation,
          tags: tags,
        };
      }
      return;
    }

    // Production: call Tauri backend
    try {
      const normalizedTags = await setTagsService(id, tags);

      // Update selected conversation with normalized tags
      if (selectedConversation && selectedConversation.id === id) {
        selectedConversation = {
          ...selectedConversation,
          tags: normalizedTags,
        };
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
        else if (selectedConversationId) {
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

    // Load tags for autocomplete (production only)
    if (!import.meta.env.DEV) {
      tagsStore.load();
    }
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
        {conversations}
        selectedId={selectedConversationId}
        onSelect={handleSelectConversation}
        onToggleBookmark={handleToggleBookmark}
        isLoading={isLoadingList}
        bind:listRef={conversationListRef}
      />
    </ErrorBoundary>

    <ErrorBoundary title="Content error" description="Failed to load the conversation details.">
      <DetailPane hasSelection={selectedConversationId !== null} isLoading={isLoadingDetail}>
        {#if selectedConversation}
          <ConversationDetail
            conversation={selectedConversation}
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
    <AnalyticsModal {conversations} />
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
