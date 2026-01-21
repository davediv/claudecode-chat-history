<script lang="ts">
  /**
   * Code block component with syntax highlighting.
   *
   * Features:
   * - Shiki-powered syntax highlighting
   * - Language label in header
   * - Copy button with feedback
   * - Horizontal scroll for long lines
   * - Graceful fallback for unknown languages
   */
  import { codeToHtml } from "shiki/bundle/web";
  import { copyToClipboard } from "$lib/services";

  interface Props {
    /** Code content to highlight */
    code: string;
    /** Programming language for syntax highlighting */
    language?: string;
    /** Handler called when code is copied */
    onCopy?: (success: boolean) => void;
  }

  let { code, language = "text", onCopy }: Props = $props();

  let highlightedHtml = $state<string>("");
  let isCopied = $state(false);
  let isLoading = $state(true);

  /**
   * Map short language names to full names for display.
   */
  const languageLabels: Record<string, string> = {
    js: "JavaScript",
    javascript: "JavaScript",
    ts: "TypeScript",
    typescript: "TypeScript",
    tsx: "TypeScript",
    jsx: "JavaScript",
    py: "Python",
    python: "Python",
    rs: "Rust",
    rust: "Rust",
    go: "Go",
    html: "HTML",
    css: "CSS",
    scss: "SCSS",
    json: "JSON",
    bash: "Bash",
    sh: "Shell",
    shell: "Shell",
    sql: "SQL",
    md: "Markdown",
    markdown: "Markdown",
    yaml: "YAML",
    yml: "YAML",
    svelte: "Svelte",
    vue: "Vue",
    xml: "XML",
    c: "C",
    cpp: "C++",
    "c++": "C++",
    java: "Java",
    kotlin: "Kotlin",
    swift: "Swift",
    ruby: "Ruby",
    php: "PHP",
    toml: "TOML",
    dockerfile: "Dockerfile",
    graphql: "GraphQL",
    text: "Plain Text",
    plaintext: "Plain Text",
  };

  /**
   * Normalize language name for Shiki.
   */
  function normalizeLanguage(lang: string): string {
    const lower = lang.toLowerCase();
    const aliases: Record<string, string> = {
      js: "javascript",
      ts: "typescript",
      py: "python",
      rs: "rust",
      sh: "bash",
      shell: "bash",
      yml: "yaml",
      md: "markdown",
      "c++": "cpp",
    };
    return aliases[lower] || lower;
  }

  /**
   * Get display label for language.
   */
  function getLanguageLabel(lang: string): string {
    const lower = lang.toLowerCase();
    return languageLabels[lower] || lang.charAt(0).toUpperCase() + lang.slice(1);
  }

  /**
   * Highlight code with Shiki.
   */
  async function highlightCode(code: string, lang: string): Promise<string> {
    try {
      const normalizedLang = normalizeLanguage(lang);
      const html = await codeToHtml(code, {
        lang: normalizedLang,
        theme: "github-dark",
      });
      return html;
    } catch {
      // Fallback to plain text on error
      const escaped = code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
      return `<pre class="shiki" style="background-color:#0d1117"><code>${escaped}</code></pre>`;
    }
  }

  /**
   * Copy code to clipboard using Tauri clipboard API.
   */
  async function handleCopy() {
    const success = await copyToClipboard(code);
    if (success) {
      isCopied = true;
      onCopy?.(true);
      setTimeout(() => {
        isCopied = false;
      }, 2000);
    } else {
      onCopy?.(false);
    }
  }

  // Highlight code when component mounts or code/language changes
  $effect(() => {
    isLoading = true;
    highlightCode(code, language).then((html) => {
      highlightedHtml = html;
      isLoading = false;
    });
  });
</script>

<div class="code-block">
  <div class="code-header">
    <span class="code-language">{getLanguageLabel(language)}</span>
    <button
      class="copy-button"
      onclick={handleCopy}
      aria-label={isCopied ? "Copied!" : "Copy code"}
      title={isCopied ? "Copied!" : "Copy to clipboard"}
    >
      {#if isCopied}
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        <span class="copy-text">Copied!</span>
      {:else}
        <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <span class="copy-text">Copy</span>
      {/if}
    </button>
  </div>

  <div class="code-content">
    {#if isLoading}
      <pre class="code-fallback"><code>{code}</code></pre>
    {:else}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html highlightedHtml}
    {/if}
  </div>
</div>

<style>
  .code-block {
    background-color: #0d1117;
    border-radius: 8px;
    overflow: hidden;
    font-family: "SF Mono", Monaco, Menlo, Consolas, "Liberation Mono", "Courier New", monospace;
  }

  .code-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background-color: #161b22;
    border-bottom: 1px solid #30363d;
    font-size: 0.75rem;
  }

  .code-language {
    color: #8b949e;
    font-weight: 500;
    font-family: inherit;
  }

  .copy-button {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    background-color: transparent;
    border: 1px solid #30363d;
    border-radius: 4px;
    color: #8b949e;
    cursor: pointer;
    font-size: 0.6875rem;
    font-family: inherit;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease,
      color 0.15s ease;
  }

  .copy-button:hover {
    background-color: #21262d;
    border-color: #8b949e;
    color: #c9d1d9;
  }

  .copy-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .icon {
    width: 0.875rem;
    height: 0.875rem;
  }

  .copy-text {
    font-family:
      Inter,
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      sans-serif;
  }

  .code-content {
    overflow-x: auto;
    max-height: 500px;
    overflow-y: auto;
  }

  .code-content :global(pre) {
    margin: 0;
    padding: 0.75rem 1rem;
    overflow-x: auto;
  }

  .code-content :global(code) {
    font-family: inherit;
    font-size: 0.8125rem;
    line-height: 1.5;
    tab-size: 2;
  }

  .code-content :global(.shiki) {
    background-color: #0d1117 !important;
  }

  .code-fallback {
    margin: 0;
    padding: 0.75rem 1rem;
    background-color: #0d1117;
    color: #c9d1d9;
  }

  .code-fallback code {
    font-family: inherit;
    font-size: 0.8125rem;
    line-height: 1.5;
    white-space: pre;
  }
</style>
