/**
 * Tests for ConversationCard component.
 *
 * Covers:
 * - Rendering with various props
 * - User interactions (click, keyboard)
 * - Accessibility attributes
 * - Date formatting logic
 * - Preview truncation
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import ConversationCard from "./ConversationCard.svelte";

describe("ConversationCard", () => {
  const defaultProps = {
    id: "conv-123",
    projectName: "my-project",
    preview: "This is a preview of the conversation content",
    lastTime: new Date().toISOString(),
    messageCount: 5,
    bookmarked: false,
    isSelected: false,
  };

  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2025-01-21T12:00:00Z"));
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  // ========== Rendering Tests ==========

  describe("rendering", () => {
    it("renders with required props", () => {
      render(ConversationCard, { props: { id: "conv-1" } });

      expect(screen.getByRole("option")).toBeInTheDocument();
      expect(screen.getByText("Unknown project")).toBeInTheDocument();
      expect(screen.getByText("No preview available")).toBeInTheDocument();
    });

    it("renders project name", () => {
      render(ConversationCard, { props: { ...defaultProps } });

      expect(screen.getByText("my-project")).toBeInTheDocument();
    });

    it("renders preview text", () => {
      render(ConversationCard, { props: { ...defaultProps } });

      expect(screen.getByText(defaultProps.preview)).toBeInTheDocument();
    });

    it("renders message count", () => {
      render(ConversationCard, { props: { ...defaultProps } });

      expect(screen.getByText("5")).toBeInTheDocument();
    });

    it("truncates long preview to 100 characters", () => {
      const longPreview = "A".repeat(150);
      render(ConversationCard, {
        props: { ...defaultProps, preview: longPreview },
      });

      // Should show 100 chars + ellipsis
      const previewElement = screen.getByTitle(longPreview);
      expect(previewElement.textContent).toHaveLength(101); // 100 + "…"
      expect(previewElement.textContent).toContain("…");
    });

    it("shows placeholder styling when preview is empty string", () => {
      render(ConversationCard, {
        props: { ...defaultProps, preview: "" },
      });

      // The preview element should have the placeholder class and show fallback text
      const previewElement = document.querySelector(".preview");
      expect(previewElement).toBeInTheDocument();
      expect(previewElement).toHaveClass("placeholder");
      expect(previewElement?.textContent).toBe("No preview available");
    });
  });

  // ========== Date Formatting Tests ==========

  describe("date formatting", () => {
    it('shows "just now" for recent timestamps', () => {
      const now = new Date("2025-01-21T12:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: now.toISOString() },
      });

      expect(screen.getByText("just now")).toBeInTheDocument();
    });

    it("shows minutes ago for timestamps within an hour", () => {
      const thirtyMinsAgo = new Date("2025-01-21T11:30:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: thirtyMinsAgo.toISOString() },
      });

      expect(screen.getByText("30 minutes ago")).toBeInTheDocument();
    });

    it('shows "1 minute ago" for singular', () => {
      const oneMinAgo = new Date("2025-01-21T11:59:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: oneMinAgo.toISOString() },
      });

      expect(screen.getByText("1 minute ago")).toBeInTheDocument();
    });

    it("shows hours ago for timestamps within a day", () => {
      const threeHoursAgo = new Date("2025-01-21T09:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: threeHoursAgo.toISOString() },
      });

      expect(screen.getByText("3 hours ago")).toBeInTheDocument();
    });

    it('shows "1 hour ago" for singular', () => {
      const oneHourAgo = new Date("2025-01-21T11:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: oneHourAgo.toISOString() },
      });

      expect(screen.getByText("1 hour ago")).toBeInTheDocument();
    });

    it('shows "yesterday" for previous day', () => {
      const yesterday = new Date("2025-01-20T12:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: yesterday.toISOString() },
      });

      expect(screen.getByText("yesterday")).toBeInTheDocument();
    });

    it("shows days ago for 2-6 days old", () => {
      const threeDaysAgo = new Date("2025-01-18T12:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: threeDaysAgo.toISOString() },
      });

      expect(screen.getByText("3 days ago")).toBeInTheDocument();
    });

    it("shows absolute date for 7+ days old", () => {
      const twoWeeksAgo = new Date("2025-01-07T12:00:00Z");
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: twoWeeksAgo.toISOString() },
      });

      expect(screen.getByText("Jan 7, 2025")).toBeInTheDocument();
    });

    it('shows "Unknown date" when lastTime is missing', () => {
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: undefined },
      });

      expect(screen.getByText("Unknown date")).toBeInTheDocument();
    });

    it('shows "Invalid Date" for malformed ISO date string', () => {
      render(ConversationCard, {
        props: { ...defaultProps, lastTime: "not-a-real-date" },
      });

      // JavaScript Date constructor with invalid string creates NaN date
      // which results in "Invalid Date" when formatting
      const timestamp = document.querySelector(".timestamp");
      expect(timestamp).toBeInTheDocument();
      expect(timestamp?.textContent).toBe("Invalid Date");
    });
  });

  // ========== Accessibility Tests ==========

  describe("accessibility", () => {
    it("has correct role attribute", () => {
      render(ConversationCard, { props: defaultProps });

      expect(screen.getByRole("option")).toBeInTheDocument();
    });

    it("has aria-selected attribute based on isSelected prop", () => {
      const { rerender } = render(ConversationCard, {
        props: { ...defaultProps, isSelected: false },
      });

      expect(screen.getByRole("option")).toHaveAttribute("aria-selected", "false");

      rerender({ ...defaultProps, isSelected: true });
      expect(screen.getByRole("option")).toHaveAttribute("aria-selected", "true");
    });

    it("is keyboard focusable with tabindex", () => {
      render(ConversationCard, { props: defaultProps });

      expect(screen.getByRole("option")).toHaveAttribute("tabindex", "0");
    });

    it("bookmark button has correct aria-label when not bookmarked", () => {
      render(ConversationCard, {
        props: { ...defaultProps, bookmarked: false },
      });

      expect(screen.getByRole("button")).toHaveAttribute("aria-label", "Bookmark conversation");
    });

    it("bookmark button has correct aria-label when bookmarked", () => {
      render(ConversationCard, {
        props: { ...defaultProps, bookmarked: true },
      });

      expect(screen.getByRole("button")).toHaveAttribute("aria-label", "Remove bookmark");
    });

    it("bookmark button has aria-pressed attribute", () => {
      const { rerender } = render(ConversationCard, {
        props: { ...defaultProps, bookmarked: false },
      });

      expect(screen.getByRole("button")).toHaveAttribute("aria-pressed", "false");

      rerender({ ...defaultProps, bookmarked: true });
      expect(screen.getByRole("button")).toHaveAttribute("aria-pressed", "true");
    });
  });

  // ========== User Interaction Tests ==========

  describe("user interactions", () => {
    it("calls onSelect when card is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSelect = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onSelect },
      });

      await user.click(screen.getByRole("option"));

      expect(onSelect).toHaveBeenCalledWith("conv-123");
      expect(onSelect).toHaveBeenCalledTimes(1);
    });

    it("calls onSelect when Enter key is pressed", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSelect = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onSelect },
      });

      const card = screen.getByRole("option");
      card.focus();
      await user.keyboard("{Enter}");

      expect(onSelect).toHaveBeenCalledWith("conv-123");
    });

    it("calls onSelect when Space key is pressed", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSelect = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onSelect },
      });

      const card = screen.getByRole("option");
      card.focus();
      await user.keyboard(" ");

      expect(onSelect).toHaveBeenCalledWith("conv-123");
    });

    it("calls onToggleBookmark when bookmark button is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onToggleBookmark = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onToggleBookmark },
      });

      await user.click(screen.getByRole("button"));

      expect(onToggleBookmark).toHaveBeenCalledWith("conv-123");
    });

    it("does not call onSelect when bookmark button is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSelect = vi.fn();
      const onToggleBookmark = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onSelect, onToggleBookmark },
      });

      await user.click(screen.getByRole("button"));

      expect(onToggleBookmark).toHaveBeenCalled();
      expect(onSelect).not.toHaveBeenCalled();
    });

    it("handles keyboard activation of bookmark button", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onToggleBookmark = vi.fn();
      const onSelect = vi.fn();

      render(ConversationCard, {
        props: { ...defaultProps, onToggleBookmark, onSelect },
      });

      const button = screen.getByRole("button");
      button.focus();
      await user.keyboard("{Enter}");

      expect(onToggleBookmark).toHaveBeenCalledWith("conv-123");
      expect(onSelect).not.toHaveBeenCalled();
    });
  });

  // ========== Visual State Tests ==========

  describe("visual states", () => {
    it("applies selected class when isSelected is true", () => {
      render(ConversationCard, {
        props: { ...defaultProps, isSelected: true },
      });

      expect(screen.getByRole("option")).toHaveClass("selected");
    });

    it("does not apply selected class when isSelected is false", () => {
      render(ConversationCard, {
        props: { ...defaultProps, isSelected: false },
      });

      expect(screen.getByRole("option")).not.toHaveClass("selected");
    });

    it("applies bookmarked class to button when bookmarked", () => {
      render(ConversationCard, {
        props: { ...defaultProps, bookmarked: true },
      });

      expect(screen.getByRole("button")).toHaveClass("bookmarked");
    });
  });
});
