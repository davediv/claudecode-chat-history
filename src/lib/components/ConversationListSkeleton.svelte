<script lang="ts">
  /**
   * Loading skeleton for conversation list.
   *
   * Shows animated placeholder cards during initial data load.
   */

  interface Props {
    /** Number of skeleton items to show */
    count?: number;
  }

  let { count = 8 }: Props = $props();
</script>

<div class="skeleton-container" role="status" aria-label="Loading conversations">
  {#each Array.from({ length: count }, (_, i) => i) as i (i)}
    <div class="skeleton-card">
      <div class="skeleton-header">
        <div class="skeleton-project"></div>
        <div class="skeleton-time"></div>
      </div>
      <div class="skeleton-preview"></div>
      <div class="skeleton-preview short"></div>
      <div class="skeleton-footer">
        <div class="skeleton-count"></div>
      </div>
    </div>
  {/each}
</div>

<style>
  .skeleton-container {
    padding: 0;
  }

  .skeleton-card {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .skeleton-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .skeleton-project {
    width: 60%;
    height: 0.875rem;
    background: linear-gradient(
      90deg,
      var(--color-bg-tertiary) 25%,
      var(--color-border) 50%,
      var(--color-bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 4px;
  }

  .skeleton-time {
    width: 15%;
    height: 0.75rem;
    background: linear-gradient(
      90deg,
      var(--color-bg-tertiary) 25%,
      var(--color-border) 50%,
      var(--color-bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    animation-delay: 0.1s;
    border-radius: 4px;
  }

  .skeleton-preview {
    width: 100%;
    height: 0.8125rem;
    margin-bottom: 0.375rem;
    background: linear-gradient(
      90deg,
      var(--color-bg-tertiary) 25%,
      var(--color-border) 50%,
      var(--color-bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    animation-delay: 0.2s;
    border-radius: 4px;
  }

  .skeleton-preview.short {
    width: 70%;
    animation-delay: 0.3s;
  }

  .skeleton-footer {
    margin-top: 0.375rem;
  }

  .skeleton-count {
    width: 10%;
    height: 0.6875rem;
    background: linear-gradient(
      90deg,
      var(--color-bg-tertiary) 25%,
      var(--color-border) 50%,
      var(--color-bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    animation-delay: 0.4s;
    border-radius: 4px;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>
