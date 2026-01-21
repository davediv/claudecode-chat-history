/**
 * Vitest setup file for Svelte component tests.
 *
 * Configures testing environment with:
 * - jest-dom matchers for DOM assertions
 * - Mocks for Tauri APIs
 * - Mocks for external dependencies (Shiki)
 */
import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri clipboard plugin
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
}));

// Mock Shiki syntax highlighter
vi.mock("shiki/bundle/web", () => ({
  codeToHtml: vi.fn().mockImplementation((code: string, options: { lang: string }) => {
    const escaped = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return Promise.resolve(
      `<pre class="shiki" data-language="${options.lang}" style="background-color:#0d1117"><code>${escaped}</code></pre>`
    );
  }),
}));

// Set up CSS variables that components expect using safe DOM methods
const style = document.createElement("style");
style.textContent = `
  :root {
    --color-bg-primary: #0d1117;
    --color-bg-secondary: #161b22;
    --color-bg-tertiary: #21262d;
    --color-text-primary: #c9d1d9;
    --color-text-secondary: #8b949e;
    --color-text-muted: #6e7681;
    --color-border: #30363d;
    --color-accent: #58a6ff;
    --color-accent-rgb: 88, 166, 255;
  }
`;
document.head.appendChild(style);
