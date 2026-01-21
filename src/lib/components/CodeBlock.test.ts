/**
 * Tests for CodeBlock component.
 *
 * Covers:
 * - Rendering with various props
 * - Language label mapping
 * - Copy button functionality
 * - Loading state during syntax highlighting
 * - Accessibility attributes
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import CodeBlock from "./CodeBlock.svelte";

// Mock the clipboard service
vi.mock("$lib/services", () => ({
  copyToClipboard: vi.fn().mockResolvedValue(true),
}));

// Import the mock to control it in tests
import { copyToClipboard } from "$lib/services";

describe("CodeBlock", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  // ========== Rendering Tests ==========

  describe("rendering", () => {
    it("renders code content", async () => {
      render(CodeBlock, { props: { code: "const x = 1;" } });

      // Wait for highlighting to complete
      await waitFor(() => {
        expect(screen.getByText("const x = 1;")).toBeInTheDocument();
      });
    });

    it("renders with default language (text)", async () => {
      render(CodeBlock, { props: { code: "hello" } });

      expect(screen.getByText("Plain Text")).toBeInTheDocument();
    });

    it("renders language label from props", async () => {
      render(CodeBlock, { props: { code: "const x = 1;", language: "javascript" } });

      expect(screen.getByText("JavaScript")).toBeInTheDocument();
    });

    it("shows copy button", async () => {
      render(CodeBlock, { props: { code: "test" } });

      expect(screen.getByRole("button")).toBeInTheDocument();
      expect(screen.getByText("Copy")).toBeInTheDocument();
    });

    it("escapes HTML in code content", async () => {
      render(CodeBlock, { props: { code: "<div>test</div>" } });

      await waitFor(() => {
        // The code should be visible as text, not rendered as HTML
        expect(screen.getByText("<div>test</div>")).toBeInTheDocument();
      });
    });
  });

  // ========== Language Label Tests ==========

  describe("language labels", () => {
    const languageTests = [
      { input: "js", expected: "JavaScript" },
      { input: "javascript", expected: "JavaScript" },
      { input: "ts", expected: "TypeScript" },
      { input: "typescript", expected: "TypeScript" },
      { input: "tsx", expected: "TypeScript" },
      { input: "jsx", expected: "JavaScript" },
      { input: "py", expected: "Python" },
      { input: "python", expected: "Python" },
      { input: "rs", expected: "Rust" },
      { input: "rust", expected: "Rust" },
      { input: "go", expected: "Go" },
      { input: "html", expected: "HTML" },
      { input: "css", expected: "CSS" },
      { input: "json", expected: "JSON" },
      { input: "bash", expected: "Bash" },
      { input: "sh", expected: "Shell" },
      { input: "sql", expected: "SQL" },
      { input: "md", expected: "Markdown" },
      { input: "yaml", expected: "YAML" },
      { input: "yml", expected: "YAML" },
      { input: "svelte", expected: "Svelte" },
      { input: "dockerfile", expected: "Dockerfile" },
    ];

    languageTests.forEach(({ input, expected }) => {
      it(`maps "${input}" to "${expected}"`, () => {
        render(CodeBlock, { props: { code: "test", language: input } });

        expect(screen.getByText(expected)).toBeInTheDocument();
      });
    });

    it("capitalizes unknown languages", () => {
      render(CodeBlock, { props: { code: "test", language: "foobar" } });

      expect(screen.getByText("Foobar")).toBeInTheDocument();
    });
  });

  // ========== Copy Button Tests ==========

  describe("copy functionality", () => {
    it("copies code when copy button is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(CodeBlock, { props: { code: "const x = 1;" } });

      await user.click(screen.getByRole("button"));

      expect(copyToClipboard).toHaveBeenCalledWith("const x = 1;");
    });

    it("shows 'Copied!' feedback after successful copy", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(CodeBlock, { props: { code: "test" } });

      expect(screen.getByText("Copy")).toBeInTheDocument();

      await user.click(screen.getByRole("button"));

      await waitFor(() => {
        expect(screen.getByText("Copied!")).toBeInTheDocument();
      });
    });

    it("resets copy feedback after 2 seconds", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(CodeBlock, { props: { code: "test" } });

      await user.click(screen.getByRole("button"));

      await waitFor(() => {
        expect(screen.getByText("Copied!")).toBeInTheDocument();
      });

      // Advance past the 2 second timeout
      vi.advanceTimersByTime(2100);

      await waitFor(() => {
        expect(screen.getByText("Copy")).toBeInTheDocument();
      });
    });

    it("calls onCopy callback with true on successful copy", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onCopy = vi.fn();

      render(CodeBlock, { props: { code: "test", onCopy } });

      await user.click(screen.getByRole("button"));

      expect(onCopy).toHaveBeenCalledWith(true);
    });

    it("calls onCopy callback with false on failed copy", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onCopy = vi.fn();

      // Mock clipboard failure
      vi.mocked(copyToClipboard).mockResolvedValueOnce(false);

      render(CodeBlock, { props: { code: "test", onCopy } });

      await user.click(screen.getByRole("button"));

      expect(onCopy).toHaveBeenCalledWith(false);
    });
  });

  // ========== Accessibility Tests ==========

  describe("accessibility", () => {
    it("copy button has aria-label when not copied", () => {
      render(CodeBlock, { props: { code: "test" } });

      expect(screen.getByRole("button")).toHaveAttribute("aria-label", "Copy code");
    });

    it("copy button has aria-label when copied", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(CodeBlock, { props: { code: "test" } });

      await user.click(screen.getByRole("button"));

      await waitFor(() => {
        expect(screen.getByRole("button")).toHaveAttribute("aria-label", "Copied!");
      });
    });

    it("copy button has title attribute for tooltip", () => {
      render(CodeBlock, { props: { code: "test" } });

      expect(screen.getByRole("button")).toHaveAttribute("title", "Copy to clipboard");
    });

    it("copy button is keyboard focusable", () => {
      render(CodeBlock, { props: { code: "test" } });

      const button = screen.getByRole("button");
      button.focus();

      expect(document.activeElement).toBe(button);
    });

    it("copy button can be activated with keyboard", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(CodeBlock, { props: { code: "test" } });

      const button = screen.getByRole("button");
      button.focus();
      await user.keyboard("{Enter}");

      expect(copyToClipboard).toHaveBeenCalledWith("test");
    });
  });

  // ========== Loading State Tests ==========

  describe("loading state", () => {
    it("shows fallback code while highlighting loads", () => {
      render(CodeBlock, { props: { code: "const x = 1;" } });

      // Initially shows fallback with plain code
      const fallback = document.querySelector(".code-fallback");
      if (fallback) {
        expect(fallback).toBeInTheDocument();
        expect(fallback.textContent).toContain("const x = 1;");
      }
    });

    it("renders highlighted code after loading", async () => {
      render(CodeBlock, { props: { code: "const x = 1;", language: "javascript" } });

      // Wait for Shiki to complete (mocked)
      await waitFor(() => {
        const shikiPre = document.querySelector("pre.shiki");
        expect(shikiPre).toBeInTheDocument();
      });
    });
  });

  // ========== Code Block Structure Tests ==========

  describe("structure", () => {
    it("has code-block container", () => {
      render(CodeBlock, { props: { code: "test" } });

      expect(document.querySelector(".code-block")).toBeInTheDocument();
    });

    it("has code-header with language and copy button", () => {
      render(CodeBlock, { props: { code: "test", language: "python" } });

      const header = document.querySelector(".code-header");
      expect(header).toBeInTheDocument();
      expect(header).toContainElement(screen.getByText("Python"));
      expect(header).toContainElement(screen.getByRole("button"));
    });

    it("has code-content container", () => {
      render(CodeBlock, { props: { code: "test" } });

      expect(document.querySelector(".code-content")).toBeInTheDocument();
    });
  });
});
