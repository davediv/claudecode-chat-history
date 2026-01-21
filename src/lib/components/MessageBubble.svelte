<script lang="ts">
  /**
   * Message bubble component for displaying a single message.
   *
   * Features:
   * - Role-based styling (user, assistant, system)
   * - Timestamp display (optional toggle)
   * - Content blocks rendered inline
   * - Accessible with ARIA labels
   */
  import type { Message, ContentBlock } from "$lib/types";
  import CodeBlock from "./CodeBlock.svelte";
  import { toast } from "$lib/stores/toast.svelte";

  interface Props {
    /** The message to display */
    message: Message;
    /** Whether to show the timestamp */
    showTimestamp?: boolean;
  }

  let { message, showTimestamp = true }: Props = $props();

  // Graceful handling of missing content
  const hasContent = $derived(
    message.content && Array.isArray(message.content) && message.content.length > 0
  );

  /**
   * Format message timestamp for display.
   */
  function formatTime(isoString: string | undefined): string {
    if (!isoString) return "";
    try {
      const date = new Date(isoString);
      return date.toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
      });
    } catch {
      return "";
    }
  }

  /**
   * Get role display label.
   */
  function getRoleLabel(role: Message["role"]): string {
    switch (role) {
      case "user":
        return "You";
      case "assistant":
        return "Claude";
      case "system":
        return "System";
      default:
        return role;
    }
  }

  /**
   * Check if a content block is a code block.
   */
  function isCodeBlock(block: ContentBlock): boolean {
    return block.type === "code";
  }

  /**
   * Check if a content block is a tool block.
   */
  function isToolBlock(block: ContentBlock): boolean {
    return block.type === "tool_use" || block.type === "tool_result";
  }

  /**
   * Handle code copy feedback.
   */
  function handleCodeCopy(success: boolean) {
    if (success) {
      toast.success("Copied to clipboard!");
    } else {
      toast.error("Failed to copy to clipboard");
    }
  }
</script>

<article class="message message-{message.role}" aria-label="{getRoleLabel(message.role)} message">
  <div class="message-header">
    <span class="message-role">{getRoleLabel(message.role)}</span>
    {#if showTimestamp}
      <span class="message-time">{formatTime(message.timestamp)}</span>
    {/if}
  </div>

  <div class="message-content">
    {#if hasContent}
      {#each message.content as block, index (index)}
        {#if block.type === "text"}
          <div class="content-text">{block.content || ""}</div>
        {:else if isCodeBlock(block)}
          <div class="content-code">
            <CodeBlock
              code={block.content || ""}
              language={block.language}
              onCopy={handleCodeCopy}
            />
          </div>
        {:else if isToolBlock(block)}
          <div class="content-tool">
            <div class="tool-header">
              <span class="tool-icon" aria-hidden="true">
                {#if block.type === "tool_use"}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path
                      d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
                    ></path>
                  </svg>
                {:else}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"></polyline>
                  </svg>
                {/if}
              </span>
              <span class="tool-name"
                >{block.toolName || (block.type === "tool_use" ? "Tool Call" : "Tool Result")}</span
              >
            </div>
            <pre class="tool-content"><code>{block.content || ""}</code></pre>
          </div>
        {/if}
      {/each}
    {:else}
      <div class="content-empty">No content available</div>
    {/if}
  </div>
</article>

<style>
  .message {
    max-width: 85%;
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    animation: messageSlideIn 0.2s ease-out;
  }

  @keyframes messageSlideIn {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message-user {
    margin-left: auto;
    background-color: var(--color-accent);
    color: white;
  }

  .message-user .message-role,
  .message-user .message-time {
    color: rgba(255, 255, 255, 0.8);
  }

  .message-assistant {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .message-system {
    max-width: 100%;
    margin-left: auto;
    margin-right: auto;
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.8125rem;
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.375rem;
    font-size: 0.75rem;
  }

  .message-role {
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .message-assistant .message-role {
    color: var(--color-accent);
  }

  .message-time {
    color: var(--color-text-muted);
    font-size: 0.6875rem;
  }

  .message-content {
    font-size: 0.875rem;
    line-height: 1.5;
  }

  /* Empty content state */
  .content-empty {
    color: var(--color-text-muted);
    font-style: italic;
    font-size: 0.8125rem;
  }

  /* Text content */
  .content-text {
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .content-text + .content-text,
  .content-text + .content-code,
  .content-text + .content-tool,
  .content-code + .content-text,
  .content-code + .content-code,
  .content-code + .content-tool,
  .content-tool + .content-text,
  .content-tool + .content-code,
  .content-tool + .content-tool {
    margin-top: 0.75rem;
  }

  /* Code content - wrapper for CodeBlock component */
  .content-code {
    border-radius: 8px;
    overflow: hidden;
  }

  /* Tool content */
  .content-tool {
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .message-user .content-tool {
    background-color: rgba(0, 0, 0, 0.15);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .tool-header {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    background-color: var(--color-bg-tertiary);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.6875rem;
  }

  .message-user .tool-header {
    background-color: rgba(0, 0, 0, 0.1);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }

  .tool-icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tool-icon svg {
    width: 0.75rem;
    height: 0.75rem;
    color: var(--color-text-muted);
  }

  .message-user .tool-icon svg {
    color: rgba(255, 255, 255, 0.7);
  }

  .tool-name {
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .message-user .tool-name {
    color: rgba(255, 255, 255, 0.7);
  }

  .tool-content {
    margin: 0;
    padding: 0.5rem 0.75rem;
    overflow-x: auto;
    font-family: "SF Mono", Monaco, Menlo, Consolas, monospace;
    font-size: 0.75rem;
    line-height: 1.4;
    max-height: 200px;
    overflow-y: auto;
  }

  .tool-content code {
    color: var(--color-text-secondary);
  }

  .message-user .tool-content code {
    color: rgba(255, 255, 255, 0.85);
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .message {
      animation: none;
    }
  }
</style>
