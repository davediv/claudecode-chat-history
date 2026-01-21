/**
 * Tests for SearchInput component.
 *
 * Covers:
 * - Rendering with various props
 * - Debounced search callback
 * - Keyboard shortcuts (/, Escape)
 * - Clear button functionality
 * - Loading state indicator
 * - Accessibility attributes
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import SearchInput from "./SearchInput.svelte";

describe("SearchInput", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
    cleanup();
    // Clean up any extra elements added to body
    while (document.body.firstChild) {
      document.body.removeChild(document.body.firstChild);
    }
  });

  /**
   * Helper to get the search input element specifically
   * (avoids conflicts with other input elements in the DOM)
   */
  function getSearchInput(): HTMLInputElement {
    return screen.getByPlaceholderText(/search/i) as HTMLInputElement;
  }

  // ========== Rendering Tests ==========

  describe("rendering", () => {
    it("renders with default placeholder", () => {
      render(SearchInput);

      expect(screen.getByPlaceholderText("Search conversations...")).toBeInTheDocument();
    });

    it("renders with custom placeholder", () => {
      render(SearchInput, { props: { placeholder: "Find something..." } });

      expect(screen.getByPlaceholderText("Find something...")).toBeInTheDocument();
    });

    it("shows keyboard shortcut hint when empty", () => {
      render(SearchInput);

      expect(screen.getByText("/")).toBeInTheDocument();
    });

    it("hides keyboard shortcut hint when value present", () => {
      render(SearchInput, { props: { value: "test" } });

      expect(screen.queryByText("/")).not.toBeInTheDocument();
    });

    it("shows clear button when value present", () => {
      render(SearchInput, { props: { value: "test" } });

      expect(screen.getByLabelText("Clear search")).toBeInTheDocument();
    });

    it("hides clear button when empty", () => {
      render(SearchInput, { props: { value: "" } });

      expect(screen.queryByLabelText("Clear search")).not.toBeInTheDocument();
    });

    it("shows loading indicator when isSearching is true", () => {
      render(SearchInput, { props: { isSearching: true } });

      // Loading indicator has animate element
      const svg = document.querySelector("svg.loading");
      expect(svg).toBeInTheDocument();
    });

    it("shows search icon when not searching", () => {
      render(SearchInput, { props: { isSearching: false } });

      const svg = document.querySelector("svg:not(.loading)");
      expect(svg).toBeInTheDocument();
    });
  });

  // ========== Accessibility Tests ==========

  describe("accessibility", () => {
    it("has aria-label on input", () => {
      render(SearchInput);

      expect(screen.getByRole("textbox")).toHaveAttribute("aria-label", "Search conversations");
    });

    it("has aria-busy when searching", () => {
      render(SearchInput, { props: { isSearching: true } });

      expect(screen.getByRole("textbox")).toHaveAttribute("aria-busy", "true");
    });

    it("has aria-busy false when not searching", () => {
      render(SearchInput, { props: { isSearching: false } });

      expect(screen.getByRole("textbox")).toHaveAttribute("aria-busy", "false");
    });

    it("clear button has aria-label", () => {
      render(SearchInput, { props: { value: "test" } });

      expect(screen.getByLabelText("Clear search")).toBeInTheDocument();
    });

    it("keyboard hint is hidden from screen readers", () => {
      render(SearchInput);

      const hint = screen.getByText("/");
      expect(hint).toHaveAttribute("aria-hidden", "true");
    });
  });

  // ========== Debounce Tests ==========

  describe("debouncing", () => {
    it("calls onInput immediately on typing", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onInput = vi.fn();

      render(SearchInput, { props: { onInput } });

      await user.type(screen.getByRole("textbox"), "t");

      expect(onInput).toHaveBeenCalledWith("t");
    });

    it("calls onSearch after debounce delay", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSearch = vi.fn();

      render(SearchInput, { props: { onSearch, debounceMs: 300 } });

      await user.type(screen.getByRole("textbox"), "test");

      // Should not be called immediately
      expect(onSearch).not.toHaveBeenCalled();

      // Advance timers past debounce
      vi.advanceTimersByTime(350);

      expect(onSearch).toHaveBeenCalledWith("test");
      expect(onSearch).toHaveBeenCalledTimes(1);
    });

    it("resets debounce timer on each keystroke", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSearch = vi.fn();

      render(SearchInput, { props: { onSearch, debounceMs: 300 } });

      await user.type(screen.getByRole("textbox"), "t");
      vi.advanceTimersByTime(200);

      await user.type(screen.getByRole("textbox"), "e");
      vi.advanceTimersByTime(200);

      await user.type(screen.getByRole("textbox"), "s");
      vi.advanceTimersByTime(200);

      // Still not called (200+200+200 = 600ms but each resets the 300ms timer)
      expect(onSearch).not.toHaveBeenCalled();

      await user.type(screen.getByRole("textbox"), "t");
      vi.advanceTimersByTime(350);

      expect(onSearch).toHaveBeenCalledWith("test");
      expect(onSearch).toHaveBeenCalledTimes(1);
    });

    it("uses custom debounce time", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onSearch = vi.fn();

      render(SearchInput, { props: { onSearch, debounceMs: 500 } });

      await user.type(screen.getByRole("textbox"), "test");

      vi.advanceTimersByTime(400);
      expect(onSearch).not.toHaveBeenCalled();

      vi.advanceTimersByTime(150);
      expect(onSearch).toHaveBeenCalledWith("test");
    });
  });

  // ========== Keyboard Shortcut Tests ==========

  describe("keyboard shortcuts", () => {
    it('focuses input when "/" is pressed globally', async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(SearchInput);

      const input = screen.getByRole("textbox");
      expect(document.activeElement).not.toBe(input);

      // Simulate pressing "/" on the body
      await user.keyboard("/");

      expect(document.activeElement).toBe(input);
    });

    it("clears input and blurs on Escape", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onInput = vi.fn();
      const onSearch = vi.fn();

      render(SearchInput, { props: { value: "test", onInput, onSearch } });

      const input = screen.getByRole("textbox");
      input.focus();

      await user.keyboard("{Escape}");

      expect(onInput).toHaveBeenCalledWith("");
      expect(onSearch).toHaveBeenCalledWith("");
      expect(document.activeElement).not.toBe(input);
    });

    it('does not focus input when "/" is pressed in another input', async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      // Create another input element
      const otherInput = document.createElement("input");
      otherInput.setAttribute("data-testid", "other-input");
      document.body.appendChild(otherInput);

      render(SearchInput);

      const searchInput = getSearchInput();
      otherInput.focus();

      // Type "/" in the other input
      await user.type(otherInput, "/");

      expect(document.activeElement).toBe(otherInput);
      expect(document.activeElement).not.toBe(searchInput);

      // Cleanup is handled in afterEach
    });
  });

  // ========== Clear Button Tests ==========

  describe("clear button", () => {
    it("clears input when clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const onInput = vi.fn();
      const onSearch = vi.fn();

      render(SearchInput, { props: { value: "test", onInput, onSearch } });

      await user.click(screen.getByLabelText("Clear search"));

      expect(onInput).toHaveBeenCalledWith("");
      expect(onSearch).toHaveBeenCalledWith("");
    });

    it("refocuses input after clearing", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(SearchInput, { props: { value: "test" } });

      await user.click(screen.getByLabelText("Clear search"));

      await waitFor(() => {
        expect(document.activeElement).toBe(getSearchInput());
      });
    });
  });

  // ========== Value Binding Tests ==========

  describe("value binding", () => {
    it("displays initial value", () => {
      render(SearchInput, { props: { value: "initial" } });

      expect(getSearchInput()).toHaveValue("initial");
    });

    it("updates value on typing", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });

      render(SearchInput, { props: { value: "" } });

      await user.type(getSearchInput(), "new value");

      expect(getSearchInput()).toHaveValue("new value");
    });
  });
});
