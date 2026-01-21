<script lang="ts">
  /**
   * Project filter component using FilterDropdown.
   *
   * Features:
   * - Lists all projects from get_projects command
   * - Shows conversation count per project
   * - "All Projects" option with total count
   * - Selection updates filters store
   */
  import { onMount } from "svelte";
  import FilterDropdown from "./FilterDropdown.svelte";
  import { filtersStore } from "$lib/stores";
  import { getProjects, isTauriAvailable } from "$lib/services";
  import type { ProjectInfo } from "$lib/types";

  interface Props {
    /** Optional handler called when filter changes */
    onChange?: (project: string | null) => void;
  }

  let { onChange }: Props = $props();

  let projects = $state<ProjectInfo[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Get current filter value from store
  const selectedProject = $derived(filtersStore.projectFilter ?? "");

  // Build options from projects list
  const options = $derived(
    projects.map((p) => ({
      value: p.projectName,
      label: p.projectName,
      count: p.conversationCount,
    }))
  );

  // Calculate total conversation count
  const totalCount = $derived(projects.reduce((sum, p) => sum + p.conversationCount, 0));

  async function loadProjects() {
    loading = true;
    error = null;

    try {
      if (isTauriAvailable()) {
        projects = await getProjects();
      } else {
        // Mock data for development
        console.log("[ProjectFilter] Running in browser mode, using mock data");
        projects = [
          {
            projectPath: "/mock/project-1",
            projectName: "project-1",
            conversationCount: 15,
            lastActivity: new Date().toISOString(),
          },
          {
            projectPath: "/mock/project-2",
            projectName: "project-2",
            conversationCount: 8,
            lastActivity: new Date().toISOString(),
          },
          {
            projectPath: "/mock/project-3",
            projectName: "project-3",
            conversationCount: 23,
            lastActivity: new Date().toISOString(),
          },
        ];
      }
    } catch (err) {
      console.error("Failed to load projects:", err);
      error = err instanceof Error ? err.message : "Failed to load projects";
      projects = [];
    } finally {
      loading = false;
    }
  }

  function handleChange(value: string) {
    const project = value || null; // Convert empty string to null
    filtersStore.setProject(project);
    onChange?.(project);
  }

  onMount(() => {
    loadProjects();
  });
</script>

<div class="project-filter">
  {#if loading}
    <div class="loading-placeholder">
      <span class="loading-text">Loading...</span>
    </div>
  {:else if error}
    <div class="error-state" title={error}>
      <span class="error-text">Error loading projects</span>
    </div>
  {:else}
    <FilterDropdown
      {options}
      selected={selectedProject}
      placeholder="All Projects"
      allLabel={`All Projects (${totalCount})`}
      showCounts={true}
      onChange={handleChange}
      ariaLabel="Filter by project"
    />
  {/if}
</div>

<style>
  .project-filter {
    display: inline-block;
  }

  .loading-placeholder,
  .error-state {
    display: flex;
    align-items: center;
    padding: 0.5rem 0.75rem;
    min-width: 140px;
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  .loading-text {
    color: var(--color-text-muted);
  }

  .error-state {
    border-color: var(--color-error, #ef4444);
  }

  .error-text {
    color: var(--color-error, #ef4444);
    font-size: 0.8125rem;
  }
</style>
