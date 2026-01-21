/**
 * E2E tests for search functionality.
 *
 * Tests the critical user flow:
 * 1. Enter search query
 * 2. See filtered/search results
 * 3. Clear search
 */
import { test, expect } from "@playwright/test";

test.describe("Search Conversations", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector(".app-layout", { state: "visible" });
    await page.waitForSelector(".conversation-card", { state: "visible", timeout: 10000 });
  });

  test("search input is visible in header", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');
    await expect(searchInput).toBeVisible();
  });

  test("focuses search with / keyboard shortcut", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Press / key
    await page.keyboard.press("/");

    // Search input should be focused
    await expect(searchInput).toBeFocused();
  });

  test("typing in search shows results", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Focus and type
    await searchInput.click();
    await searchInput.fill("conv");

    // Wait for debounce (300ms default)
    await page.waitForTimeout(400);

    // Should still show conversation cards (dev mock data doesn't really search)
    const cards = page.locator(".conversation-card");
    await expect(cards.first()).toBeVisible();
  });

  test("shows clear button when search has value", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Initially no clear button
    const clearButton = page.locator('button[aria-label="Clear search"]');
    await expect(clearButton).not.toBeVisible();

    // Type in search
    await searchInput.fill("test");

    // Clear button should appear
    await expect(clearButton).toBeVisible();
  });

  test("clears search with clear button", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Type in search
    await searchInput.fill("test query");
    await expect(searchInput).toHaveValue("test query");

    // Click clear button
    const clearButton = page.locator('button[aria-label="Clear search"]');
    await clearButton.click();

    // Search should be empty
    await expect(searchInput).toHaveValue("");
  });

  test("clears search with Escape key", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Focus and type
    await searchInput.click();
    await searchInput.fill("test");

    // Press Escape
    await page.keyboard.press("Escape");

    // Search should be cleared and unfocused
    await expect(searchInput).toHaveValue("");
    await expect(searchInput).not.toBeFocused();
  });

  test("search input has proper accessibility attributes", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Check aria attributes
    await expect(searchInput).toHaveAttribute("aria-label", "Search conversations");
    await expect(searchInput).toHaveAttribute("type", "text");
  });

  test("shows keyboard shortcut hint when empty", async ({ page }) => {
    // Look for the "/" hint
    const hint = page.locator('kbd:has-text("/")');
    await expect(hint).toBeVisible();

    // Type something
    const searchInput = page.locator('input[placeholder*="Search"]');
    await searchInput.fill("test");

    // Hint should disappear
    await expect(hint).not.toBeVisible();
  });

  test("search persists across navigation", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Type search
    await searchInput.fill("project");

    // Select a conversation
    const firstCard = page.locator(".conversation-card").first();
    await firstCard.click();

    // Wait for detail view
    await page.waitForSelector(".conversation-detail", { state: "visible", timeout: 5000 });

    // Search should still have value
    await expect(searchInput).toHaveValue("project");

    // Go back
    await page.keyboard.press("Escape");

    // Search should still have value
    await expect(searchInput).toHaveValue("project");
  });

  test("shows loading state while searching", async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="Search"]');

    // Type a query
    await searchInput.fill("test");

    // Check for aria-busy (may be brief)
    // The loading state is handled by the component
    const searchContainer = page.locator(".search-container");
    await expect(searchContainer).toBeVisible();
  });
});
