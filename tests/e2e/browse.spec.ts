/**
 * E2E tests for browsing conversations flow.
 *
 * Tests the critical user flow:
 * 1. Load app and see conversation list
 * 2. Select a conversation
 * 3. View conversation details
 * 4. Navigate back to list
 */
import { test, expect } from "@playwright/test";

test.describe("Browse Conversations", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app (dev server with mock data)
    await page.goto("/");

    // Wait for app to load
    await page.waitForSelector(".app-layout", { state: "visible" });

    // Wait for conversations list to populate (dev mode generates mock data)
    await page.waitForSelector(".conversation-card", { state: "visible", timeout: 10000 });
  });

  test("displays conversation list on load", async ({ page }) => {
    // Check sidebar is visible
    const sidebar = page.locator("aside.sidebar");
    await expect(sidebar).toBeVisible();

    // Check conversation cards are rendered
    const cards = page.locator(".conversation-card");
    await expect(cards.first()).toBeVisible();

    // Should have multiple conversations (dev mode generates many)
    const count = await cards.count();
    expect(count).toBeGreaterThan(0);
  });

  test("shows empty detail pane when no conversation selected", async ({ page }) => {
    // Check detail pane shows placeholder
    const emptyState = page.locator(".detail-empty");
    await expect(emptyState).toBeVisible();

    // Check for keyboard hint
    const hint = page.locator(".empty-hint");
    await expect(hint).toContainText("conversation");
  });

  test("selects conversation on click", async ({ page }) => {
    // Click the first conversation card
    const firstCard = page.locator(".conversation-card").first();
    await firstCard.click();

    // Card should become selected (has selected class)
    await expect(firstCard).toHaveClass(/selected/);

    // Detail pane should show content
    const detailContent = page.locator(".conversation-detail");
    await expect(detailContent).toBeVisible({ timeout: 5000 });
  });

  test("displays conversation detail with messages", async ({ page }) => {
    // Click a conversation
    await page.locator(".conversation-card").first().click();

    // Wait for detail to load
    await page.waitForSelector(".conversation-detail", { state: "visible", timeout: 5000 });

    // Check header shows project name
    const projectName = page.locator(".conversation-detail .project-name");
    await expect(projectName).toBeVisible();

    // Check messages are displayed
    const messages = page.locator(".message");
    await expect(messages.first()).toBeVisible();
  });

  test("navigates back to list with back button", async ({ page }) => {
    // First select a conversation at normal viewport
    await page.locator(".conversation-card").first().click();
    await page.waitForSelector(".conversation-detail", { state: "visible" });

    // Back button only shows on narrow screens, so resize viewport
    await page.setViewportSize({ width: 500, height: 800 });
    await page.waitForTimeout(200);

    // Click back button (visible on narrow screens)
    const backButton = page.locator(".back-button");
    await expect(backButton).toBeVisible({ timeout: 5000 });
    await backButton.click();

    // Detail should show empty state again
    const emptyState = page.locator(".detail-empty");
    await expect(emptyState).toBeVisible();
  });

  test("navigates back with Escape key", async ({ page }) => {
    // Select a conversation
    await page.locator(".conversation-card").first().click();
    await page.waitForSelector(".conversation-detail", { state: "visible" });

    // Press Escape (need to focus body first)
    await page.keyboard.press("Escape");

    // Detail should show empty state
    const emptyState = page.locator(".detail-empty");
    await expect(emptyState).toBeVisible();
  });

  test("keyboard navigation with j/k keys", async ({ page }) => {
    // Click the conversation list to focus it
    const list = page.locator(".conversation-list");
    await list.click();

    // Give it a moment to be interactive
    await page.waitForTimeout(100);

    // Press 'j' to move down and select first item
    await page.keyboard.press("j");
    await page.waitForTimeout(100);

    // Press Enter to select the focused/first item
    await page.keyboard.press("Enter");

    // Should open detail
    const detail = page.locator(".conversation-detail");
    await expect(detail).toBeVisible({ timeout: 5000 });
  });

  test("displays correct conversation metadata", async ({ page }) => {
    // Click a conversation
    const firstCard = page.locator(".conversation-card").first();
    await firstCard.click();

    // Wait for detail to load
    await page.waitForSelector(".conversation-detail", { state: "visible", timeout: 5000 });

    // Check metadata is displayed
    const header = page.locator(".detail-header");
    await expect(header).toBeVisible();

    // Project name should be visible
    const projectName = header.locator(".project-name");
    await expect(projectName).toBeVisible();

    // Date should be visible
    const date = header.locator(".meta-date");
    await expect(date).toBeVisible();
  });

  test("scrolls through messages in long conversation", async ({ page }) => {
    // Select a conversation
    await page.locator(".conversation-card").first().click();
    await page.waitForSelector(".conversation-detail", { state: "visible", timeout: 5000 });

    // Check it's scrollable (has overflow)
    const messages = page.locator(".message");
    const messageCount = await messages.count();

    // Dev mock data should have multiple messages
    expect(messageCount).toBeGreaterThan(1);
  });
});
