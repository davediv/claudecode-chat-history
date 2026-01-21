<script lang="ts">
  /**
   * Main application page.
   *
   * Layout:
   * - Header bar with search and filters
   * - Two-column split: sidebar (conversation list) and detail pane
   * - Error boundaries for graceful error handling
   */
  import Header from "$lib/components/Header.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DetailPane from "$lib/components/DetailPane.svelte";
  import ConversationDetail from "$lib/components/ConversationDetail.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import { ErrorBoundary } from "$lib/components";
  import type { Conversation, Message, ContentBlock } from "$lib/types";

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

  function handleSearch() {
    // Search is now handled by the searchStore in Header component
    // This callback can be used for additional page-level side effects
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
  <Header onSearch={handleSearch} />

  <div class="main-content">
    <ErrorBoundary title="Sidebar error" description="Failed to load the conversation list.">
      <Sidebar
        {conversations}
        selectedId={selectedConversationId}
        onSelect={handleSelectConversation}
        isLoading={isLoadingList}
      />
    </ErrorBoundary>

    <ErrorBoundary title="Content error" description="Failed to load the conversation details.">
      <DetailPane hasSelection={selectedConversationId !== null} isLoading={isLoadingDetail}>
        {#if selectedConversation}
          <ConversationDetail conversation={selectedConversation} onBack={handleBack} />
        {/if}
      </DetailPane>
    </ErrorBoundary>
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
