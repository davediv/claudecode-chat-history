/**
 * Tests for MessageBubble component.
 *
 * Covers:
 * - Rendering with various props
 * - Role-based styling and labels
 * - Content block type rendering
 * - Timestamp formatting
 * - Accessibility attributes
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import MessageBubble from "./MessageBubble.svelte";
import type { Message, ContentBlock } from "$lib/types";

// Mock the toast store
vi.mock("$lib/stores/toast.svelte", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

// Mock the clipboard service (used by CodeBlock)
vi.mock("$lib/services", () => ({
  copyToClipboard: vi.fn().mockResolvedValue(true),
}));

describe("MessageBubble", () => {
  // Helper to create a message with specific content
  function createMessage(
    role: Message["role"],
    content: ContentBlock[],
    timestamp?: string
  ): Message {
    return {
      id: `msg-${Date.now()}`,
      role,
      content,
      timestamp: timestamp || new Date().toISOString(),
    };
  }

  // Helper to create text content block
  function textBlock(text: string): ContentBlock {
    return { type: "text", content: text };
  }

  // Helper to create code content block
  function codeBlock(code: string, language?: string): ContentBlock {
    return { type: "code", content: code, language };
  }

  // Helper to create tool_use content block
  function toolUseBlock(content: string, toolName?: string): ContentBlock {
    return { type: "tool_use", content, toolName };
  }

  // Helper to create tool_result content block
  function toolResultBlock(content: string, toolName?: string): ContentBlock {
    return { type: "tool_result", content, toolName };
  }

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ========== Role Label Tests ==========

  describe("role labels", () => {
    it('displays "You" for user messages', () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("You")).toBeInTheDocument();
    });

    it('displays "Claude" for assistant messages', () => {
      const message = createMessage("assistant", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("Claude")).toBeInTheDocument();
    });

    it('displays "System" for system messages', () => {
      const message = createMessage("system", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("System")).toBeInTheDocument();
    });
  });

  // ========== Timestamp Tests ==========

  describe("timestamp", () => {
    it("shows timestamp by default", () => {
      const message = createMessage("user", [textBlock("Hello")], "2025-01-21T14:30:00Z");
      render(MessageBubble, { props: { message } });

      // Should show formatted time (depends on locale, but should contain time)
      const timeElement = document.querySelector(".message-time");
      expect(timeElement).toBeInTheDocument();
      expect(timeElement?.textContent).toBeTruthy();
    });

    it("hides timestamp when showTimestamp is false", () => {
      const message = createMessage("user", [textBlock("Hello")], "2025-01-21T14:30:00Z");
      render(MessageBubble, { props: { message, showTimestamp: false } });

      const timeElement = document.querySelector(".message-time");
      expect(timeElement).not.toBeInTheDocument();
    });

    it("handles missing timestamp gracefully", () => {
      const message: Message = {
        id: "msg-1",
        role: "user",
        content: [textBlock("Hello")],
        timestamp: "",
      };
      render(MessageBubble, { props: { message } });

      // Should still render without crashing
      expect(screen.getByText("You")).toBeInTheDocument();
    });

    it("handles invalid timestamp gracefully", () => {
      const message: Message = {
        id: "msg-1",
        role: "user",
        content: [textBlock("Hello")],
        timestamp: "invalid-date",
      };
      render(MessageBubble, { props: { message } });

      // Should still render without crashing
      expect(screen.getByText("You")).toBeInTheDocument();
    });
  });

  // ========== Content Block Tests ==========

  describe("content blocks", () => {
    it("renders text content blocks", () => {
      const message = createMessage("user", [textBlock("Hello, world!")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("Hello, world!")).toBeInTheDocument();
    });

    it("renders multiple text blocks", () => {
      const message = createMessage("assistant", [
        textBlock("First paragraph"),
        textBlock("Second paragraph"),
      ]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("First paragraph")).toBeInTheDocument();
      expect(screen.getByText("Second paragraph")).toBeInTheDocument();
    });

    it("renders code content blocks with CodeBlock component", () => {
      const message = createMessage("assistant", [codeBlock("const x = 1;", "javascript")]);
      render(MessageBubble, { props: { message } });

      // CodeBlock should render with language label
      expect(screen.getByText("JavaScript")).toBeInTheDocument();
    });

    it("renders tool_use blocks with tool icon", () => {
      const message = createMessage("assistant", [toolUseBlock('{"param": "value"}', "my_tool")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("my_tool")).toBeInTheDocument();
    });

    it("renders tool_result blocks with checkmark icon", () => {
      const message = createMessage("assistant", [toolResultBlock("Success!", "my_tool")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("my_tool")).toBeInTheDocument();
    });

    it("shows default tool name for tool_use without toolName", () => {
      const message = createMessage("assistant", [{ type: "tool_use" as const, content: "test" }]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("Tool Call")).toBeInTheDocument();
    });

    it("shows default tool name for tool_result without toolName", () => {
      const message = createMessage("assistant", [
        { type: "tool_result" as const, content: "test" },
      ]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("Tool Result")).toBeInTheDocument();
    });

    it("renders mixed content types", () => {
      const message = createMessage("assistant", [
        textBlock("Here is some code:"),
        codeBlock("console.log('hello');", "javascript"),
        textBlock("And here is a tool call:"),
        toolUseBlock('{"file": "test.ts"}', "read_file"),
      ]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("Here is some code:")).toBeInTheDocument();
      expect(screen.getByText("JavaScript")).toBeInTheDocument();
      expect(screen.getByText("And here is a tool call:")).toBeInTheDocument();
      expect(screen.getByText("read_file")).toBeInTheDocument();
    });
  });

  // ========== Empty Content Tests ==========

  describe("empty content", () => {
    it('shows "No content available" for empty content array', () => {
      const message = createMessage("assistant", []);
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("No content available")).toBeInTheDocument();
    });

    it("handles missing content gracefully", () => {
      const message: Message = {
        id: "msg-1",
        role: "assistant",
        content: undefined as unknown as ContentBlock[],
        timestamp: new Date().toISOString(),
      };
      render(MessageBubble, { props: { message } });

      expect(screen.getByText("No content available")).toBeInTheDocument();
    });

    it("handles empty string content in text block", () => {
      const message = createMessage("user", [textBlock("")]);
      render(MessageBubble, { props: { message } });

      // Should render but content-text should be empty
      const textContent = document.querySelector(".content-text");
      expect(textContent).toBeInTheDocument();
      expect(textContent?.textContent).toBe("");
    });
  });

  // ========== Accessibility Tests ==========

  describe("accessibility", () => {
    it("has article role with aria-label for user message", () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveAttribute("aria-label", "You message");
    });

    it("has article role with aria-label for assistant message", () => {
      const message = createMessage("assistant", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveAttribute("aria-label", "Claude message");
    });

    it("has article role with aria-label for system message", () => {
      const message = createMessage("system", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveAttribute("aria-label", "System message");
    });

    it("tool icon has aria-hidden attribute", () => {
      const message = createMessage("assistant", [toolUseBlock("test", "my_tool")]);
      render(MessageBubble, { props: { message } });

      const toolIcon = document.querySelector(".tool-icon");
      expect(toolIcon).toHaveAttribute("aria-hidden", "true");
    });
  });

  // ========== Visual State Tests ==========

  describe("visual states", () => {
    it("applies message-user class for user messages", () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveClass("message-user");
    });

    it("applies message-assistant class for assistant messages", () => {
      const message = createMessage("assistant", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveClass("message-assistant");
    });

    it("applies message-system class for system messages", () => {
      const message = createMessage("system", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(screen.getByRole("article")).toHaveClass("message-system");
    });
  });

  // ========== Content Wrapper Classes Tests ==========

  describe("content wrapper classes", () => {
    it("wraps text content in content-text class", () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(document.querySelector(".content-text")).toBeInTheDocument();
    });

    it("wraps code content in content-code class", () => {
      const message = createMessage("assistant", [codeBlock("test", "js")]);
      render(MessageBubble, { props: { message } });

      expect(document.querySelector(".content-code")).toBeInTheDocument();
    });

    it("wraps tool content in content-tool class", () => {
      const message = createMessage("assistant", [toolUseBlock("test", "tool")]);
      render(MessageBubble, { props: { message } });

      expect(document.querySelector(".content-tool")).toBeInTheDocument();
    });
  });

  // ========== Structure Tests ==========

  describe("structure", () => {
    it("has message-header with role and time", () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      const header = document.querySelector(".message-header");
      expect(header).toBeInTheDocument();
      expect(header).toContainElement(document.querySelector(".message-role"));
    });

    it("has message-content container", () => {
      const message = createMessage("user", [textBlock("Hello")]);
      render(MessageBubble, { props: { message } });

      expect(document.querySelector(".message-content")).toBeInTheDocument();
    });

    it("tool blocks have header and content sections", () => {
      const message = createMessage("assistant", [toolUseBlock("test data", "my_tool")]);
      render(MessageBubble, { props: { message } });

      expect(document.querySelector(".tool-header")).toBeInTheDocument();
      expect(document.querySelector(".tool-content")).toBeInTheDocument();
    });
  });
});
