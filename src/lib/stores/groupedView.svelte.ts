/**
 * Grouped view store using Svelte 5 Runes.
 *
 * Manages the state for grouped conversation view including:
 * - Which projects are expanded/collapsed
 * - Whether grouped view mode is enabled
 * - Persistence to localStorage
 */

import { SvelteSet } from "svelte/reactivity";

const STORAGE_KEY_EXPANDED = "claudeHistory:expandedProjects";
const STORAGE_KEY_ENABLED = "claudeHistory:groupedViewEnabled";

// Load initial state from localStorage
function loadExpandedProjects(): SvelteSet<string> {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_EXPANDED);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed)) {
        return new SvelteSet(parsed);
      }
    }
  } catch {
    // Ignore parse errors, use default
  }
  return new SvelteSet();
}

function loadGroupedViewEnabled(): boolean {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_ENABLED);
    if (stored !== null) {
      return stored === "true";
    }
  } catch {
    // Ignore parse errors, use default
  }
  // Default to grouped view enabled
  return true;
}

// Reactive state
const expandedProjects = loadExpandedProjects();
let groupedViewEnabled = $state<boolean>(loadGroupedViewEnabled());

/**
 * Save expanded projects to localStorage.
 */
function persistExpanded(): void {
  try {
    localStorage.setItem(STORAGE_KEY_EXPANDED, JSON.stringify(Array.from(expandedProjects)));
  } catch {
    // Ignore storage errors
  }
}

/**
 * Save grouped view enabled state to localStorage.
 */
function persistEnabled(): void {
  try {
    localStorage.setItem(STORAGE_KEY_ENABLED, String(groupedViewEnabled));
  } catch {
    // Ignore storage errors
  }
}

/**
 * Toggle a project's expanded/collapsed state.
 */
export function toggleProject(projectName: string): void {
  if (expandedProjects.has(projectName)) {
    expandedProjects.delete(projectName);
  } else {
    expandedProjects.add(projectName);
  }
  persistExpanded();
}

/**
 * Expand a specific project.
 */
export function expandProject(projectName: string): void {
  if (!expandedProjects.has(projectName)) {
    expandedProjects.add(projectName);
    persistExpanded();
  }
}

/**
 * Collapse a specific project.
 */
export function collapseProject(projectName: string): void {
  if (expandedProjects.has(projectName)) {
    expandedProjects.delete(projectName);
    persistExpanded();
  }
}

/**
 * Check if a project is expanded.
 */
export function isProjectExpanded(projectName: string): boolean {
  return expandedProjects.has(projectName);
}

/**
 * Expand all projects.
 */
export function expandAll(projectNames: string[]): void {
  expandedProjects.clear();
  for (const name of projectNames) {
    expandedProjects.add(name);
  }
  persistExpanded();
}

/**
 * Collapse all projects.
 */
export function collapseAll(): void {
  expandedProjects.clear();
  persistExpanded();
}

/**
 * Enable grouped view mode.
 */
export function enableGroupedView(): void {
  groupedViewEnabled = true;
  persistEnabled();
}

/**
 * Disable grouped view mode.
 */
export function disableGroupedView(): void {
  groupedViewEnabled = false;
  persistEnabled();
}

/**
 * Toggle grouped view mode.
 */
export function toggleGroupedView(): void {
  groupedViewEnabled = !groupedViewEnabled;
  persistEnabled();
}

// Export reactive getters and actions
export const groupedViewStore = {
  get expandedProjects(): SvelteSet<string> {
    return expandedProjects;
  },
  get groupedViewEnabled(): boolean {
    return groupedViewEnabled;
  },
  // Actions
  toggleProject,
  expandProject,
  collapseProject,
  isProjectExpanded,
  expandAll,
  collapseAll,
  enableGroupedView,
  disableGroupedView,
  toggleGroupedView,
};
