/**
 * Tauri IPC mock utilities for E2E tests.
 *
 * Provides functions to inject mock responses into the browser
 * by intercepting Tauri's IPC calls via page.evaluate().
 */

import type { Page } from "@playwright/test";
import { mockProjects, mockConversationSummaries, mockConversations } from "../fixtures/mock-data";

/**
 * Setup Tauri mocks on the page.
 * This injects a mock __TAURI__ object that intercepts IPC calls.
 */
export async function setupTauriMocks(page: Page): Promise<void> {
  await page.addInitScript(() => {
    // Mock data - will be injected separately
    (window as unknown as { __MOCK_DATA__: unknown }).__MOCK_DATA__ = null;

    // Create mock __TAURI__ object
    (window as unknown as { __TAURI__: unknown }).__TAURI__ = {
      core: {
        invoke: async (command: string, args?: Record<string, unknown>) => {
          interface MockSummary {
            id: string;
            projectName: string;
            firstMessage: string;
            timestamp: string;
            messageCount: number;
            isBookmarked: boolean;
          }
          const mockData = (window as unknown as { __MOCK_DATA__: unknown }).__MOCK_DATA__ as {
            projects: unknown[];
            summaries: MockSummary[];
            conversations: Record<string, unknown>;
          };

          if (!mockData) {
            console.warn("[E2E Mock] No mock data available");
            return [];
          }

          console.log("[E2E Mock] invoke:", command, args);

          switch (command) {
            case "get_projects":
              return mockData.projects;

            case "get_conversations": {
              const filters = args?.filters as { project?: string } | null;
              let summaries = [...mockData.summaries];

              if (filters?.project) {
                summaries = summaries.filter((s) => s.projectName === filters.project);
              }

              return summaries;
            }

            case "get_conversation": {
              const id = args?.id as string;
              const conv = mockData.conversations[id];
              if (!conv) {
                throw new Error(`Conversation not found: ${id}`);
              }
              return conv;
            }

            case "search_conversations": {
              const query = (args?.query as string) || "";
              if (query.length < 2) return [];

              // Simple search through mock data
              const lowerQuery = query.toLowerCase();
              const results: unknown[] = [];

              Object.values(mockData.conversations).forEach((conv: unknown) => {
                const c = conv as {
                  id: string;
                  messages: { content: { content: string }[] }[];
                };
                let matchCount = 0;
                let snippet = "";

                c.messages.forEach((msg) => {
                  msg.content.forEach((block) => {
                    if (block.content.toLowerCase().includes(lowerQuery)) {
                      matchCount++;
                      if (!snippet) {
                        const idx = block.content.toLowerCase().indexOf(lowerQuery);
                        const start = Math.max(0, idx - 30);
                        const end = Math.min(block.content.length, idx + query.length + 30);
                        snippet = block.content.slice(start, end);
                      }
                    }
                  });
                });

                if (matchCount > 0) {
                  results.push({
                    conversationId: c.id,
                    snippet,
                    matchCount,
                    rank: matchCount,
                  });
                }
              });

              return results.sort(
                (a: unknown, b: unknown) =>
                  (b as { matchCount: number }).matchCount -
                  (a as { matchCount: number }).matchCount
              );
            }

            case "toggle_bookmark": {
              const convId = args?.conversationId as string;
              const conv = mockData.conversations[convId] as { bookmarked: boolean } | undefined;
              if (conv) {
                conv.bookmarked = !conv.bookmarked;
                return conv.bookmarked;
              }
              return false;
            }

            case "set_tags":
              return args?.tags || [];

            case "get_all_tags":
              return [
                { tag: "authentication", count: 1 },
                { tag: "jwt", count: 1 },
                { tag: "ui", count: 1 },
                { tag: "database", count: 1 },
              ];

            default:
              console.warn("[E2E Mock] Unknown command:", command);
              return null;
          }
        },
      },
      event: {
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        listen: async (eventName: string, _callback: unknown) => {
          console.log("[E2E Mock] listen:", eventName);
          return () => {};
        },
        emit: async (eventName: string, payload: unknown) => {
          console.log("[E2E Mock] emit:", eventName, payload);
        },
      },
    };
  });

  // Inject mock data
  await page.addInitScript(
    ({ projects, summaries, conversations }) => {
      (window as unknown as { __MOCK_DATA__: unknown }).__MOCK_DATA__ = {
        projects,
        summaries,
        conversations,
      };
    },
    {
      projects: mockProjects,
      summaries: mockConversationSummaries,
      conversations: mockConversations,
    }
  );
}

/**
 * Mock clipboard write operation.
 * Stores the copied text for later verification.
 */
export async function setupClipboardMock(page: Page): Promise<void> {
  await page.addInitScript(() => {
    (window as unknown as { __CLIPBOARD_CONTENT__: string }).__CLIPBOARD_CONTENT__ = "";

    // Mock the Tauri clipboard plugin by exposing a mock on window
    Object.defineProperty(window, "__clipboardMock__", {
      value: {
        writeText: async (text: string) => {
          (window as unknown as { __CLIPBOARD_CONTENT__: string }).__CLIPBOARD_CONTENT__ = text;
          console.log("[E2E Mock] Clipboard write:", text.substring(0, 50) + "...");
          return Promise.resolve();
        },
      },
    });
  });
}

/**
 * Get the content that was "copied" to clipboard.
 */
export async function getClipboardContent(page: Page): Promise<string> {
  return await page.evaluate(() => {
    return (window as unknown as { __CLIPBOARD_CONTENT__: string }).__CLIPBOARD_CONTENT__ || "";
  });
}

/**
 * Wait for the app to be fully loaded.
 */
export async function waitForAppReady(page: Page): Promise<void> {
  // Wait for the main container to be visible
  await page.waitForSelector(".app-container", { state: "visible", timeout: 10000 });

  // Wait a bit for initial data load
  await page.waitForTimeout(500);
}
