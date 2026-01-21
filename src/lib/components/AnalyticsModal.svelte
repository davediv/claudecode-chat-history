<script lang="ts">
  /**
   * Analytics Modal component displaying usage statistics.
   *
   * Features:
   * - Summary stats: total conversations, total tokens
   * - Conversations over time chart
   * - Tokens by project chart
   * - Keyboard accessible (Escape to close)
   */
  import { uiStore } from "$lib/stores";

  interface ConversationSummary {
    id: string;
    projectName: string;
    lastTime: string;
    messageCount: number;
  }

  interface Props {
    /** Conversation data for analytics */
    conversations: ConversationSummary[];
  }

  let { conversations = [] }: Props = $props();

  /**
   * Compute analytics data from conversations.
   * Using a function instead of $derived with Map to avoid ESLint warnings.
   */
  function computeAnalytics(convList: ConversationSummary[]) {
    // Total counts
    const totalConversations = convList.length;

    // Group by project using object instead of Map
    const projectGroups: Record<string, { count: number; messages: number }> = {};
    for (const conv of convList) {
      const existing = projectGroups[conv.projectName] || { count: 0, messages: 0 };
      projectGroups[conv.projectName] = {
        count: existing.count + 1,
        messages: existing.messages + conv.messageCount,
      };
    }

    const projectStats = Object.entries(projectGroups)
      .map(([name, stats]) => ({ name, ...stats }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10); // Top 10 projects

    // Group by month using object instead of Map
    const monthGroups: Record<string, number> = {};
    for (const conv of convList) {
      try {
        const date = new Date(conv.lastTime);
        const monthKey = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}`;
        monthGroups[monthKey] = (monthGroups[monthKey] || 0) + 1;
      } catch {
        // Skip invalid dates
      }
    }

    const timeSeriesData = Object.entries(monthGroups)
      .map(([month, count]) => ({ month, count }))
      .sort((a, b) => a.month.localeCompare(b.month))
      .slice(-12); // Last 12 months

    // Calculate total messages
    const totalMessages = convList.reduce((sum, c) => sum + c.messageCount, 0);

    // Find max values for chart scaling
    const maxProjectCount = Math.max(...projectStats.map((p) => p.count), 1);
    const maxMonthCount = Math.max(...timeSeriesData.map((d) => d.count), 1);

    return {
      totalConversations,
      totalMessages,
      projectStats,
      timeSeriesData,
      maxProjectCount,
      maxMonthCount,
    };
  }

  // Computed analytics data
  const analytics = $derived(() => computeAnalytics(conversations));

  function formatMonth(monthKey: string): string {
    try {
      const [year, month] = monthKey.split("-");
      const date = new Date(parseInt(year), parseInt(month) - 1);
      return date.toLocaleDateString("en-US", { month: "short", year: "2-digit" });
    } catch {
      return monthKey;
    }
  }

  function handleClose() {
    uiStore.closeAnalyticsModal();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      handleClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleClose();
    }
  }
</script>

<div
  class="modal-backdrop"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
  role="dialog"
  aria-modal="true"
  aria-label="Usage Analytics"
  tabindex="-1"
>
  <div class="modal-content">
    <header class="modal-header">
      <h2 class="modal-title">Usage Analytics</h2>
      <button type="button" class="close-button" onclick={handleClose} aria-label="Close analytics">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"></path>
        </svg>
      </button>
    </header>

    <div class="modal-body">
      <!-- Summary Stats -->
      <section class="stats-section">
        <div class="stat-card">
          <span class="stat-value">{analytics().totalConversations.toLocaleString()}</span>
          <span class="stat-label">Total Conversations</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{analytics().totalMessages.toLocaleString()}</span>
          <span class="stat-label">Total Messages</span>
        </div>
        <div class="stat-card">
          <span class="stat-value">{analytics().projectStats.length.toLocaleString()}</span>
          <span class="stat-label">Projects</span>
        </div>
      </section>

      <!-- Conversations Over Time -->
      {#if analytics().timeSeriesData.length > 0}
        <section class="chart-section">
          <h3 class="chart-title">Conversations Over Time</h3>
          <div class="time-chart">
            {#each analytics().timeSeriesData as data (data.month)}
              <div class="time-bar-container" title="{formatMonth(data.month)}: {data.count}">
                <div
                  class="time-bar"
                  style="height: {(data.count / analytics().maxMonthCount) * 100}%"
                ></div>
                <span class="time-label">{formatMonth(data.month)}</span>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Top Projects -->
      {#if analytics().projectStats.length > 0}
        <section class="chart-section">
          <h3 class="chart-title">Top Projects</h3>
          <div class="project-chart">
            {#each analytics().projectStats as project (project.name)}
              <div class="project-row">
                <span class="project-name" title={project.name}>{project.name}</span>
                <div class="project-bar-container">
                  <div
                    class="project-bar"
                    style="width: {(project.count / analytics().maxProjectCount) * 100}%"
                  ></div>
                </div>
                <span class="project-count">{project.count}</span>
              </div>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .modal-content {
    background: var(--color-bg-secondary);
    border-radius: 12px;
    border: 1px solid var(--color-border);
    width: 100%;
    max-width: 700px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .modal-title {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .close-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-muted);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .close-button:hover {
    background: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .close-button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 1px;
  }

  .close-button svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  .modal-body {
    padding: 1.25rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* Summary Stats */
  .stats-section {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    background: var(--color-bg-tertiary);
    border-radius: 8px;
    text-align: center;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary);
    line-height: 1.2;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-top: 0.25rem;
  }

  /* Chart Sections */
  .chart-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .chart-title {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  /* Time Series Chart */
  .time-chart {
    display: flex;
    align-items: flex-end;
    gap: 0.25rem;
    height: 120px;
    padding: 0.5rem;
    background: var(--color-bg-tertiary);
    border-radius: 8px;
  }

  .time-bar-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
    min-width: 0;
  }

  .time-bar {
    width: 100%;
    max-width: 2rem;
    background: var(--color-accent);
    border-radius: 2px 2px 0 0;
    transition: height 0.3s ease;
    min-height: 2px;
  }

  .time-label {
    font-size: 0.625rem;
    color: var(--color-text-muted);
    margin-top: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  /* Project Chart */
  .project-chart {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .project-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .project-name {
    flex: 0 0 120px;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-bar-container {
    flex: 1;
    height: 1.25rem;
    background: var(--color-bg-tertiary);
    border-radius: 4px;
    overflow: hidden;
  }

  .project-bar {
    height: 100%;
    background: var(--color-accent);
    border-radius: 4px;
    transition: width 0.3s ease;
    min-width: 4px;
  }

  .project-count {
    flex: 0 0 2.5rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
    text-align: right;
  }

  /* Responsive */
  @media (max-width: 480px) {
    .stats-section {
      grid-template-columns: 1fr;
    }

    .project-name {
      flex: 0 0 80px;
    }

    .time-label {
      display: none;
    }
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    .close-button,
    .time-bar,
    .project-bar {
      transition: none;
    }
  }
</style>
