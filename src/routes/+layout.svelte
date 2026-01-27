<script lang="ts">
  import "../app.css";
  import { onMount, onDestroy } from "svelte";
  import { listenToConversationsUpdated, type UnlistenFn } from "$lib/services/tauri";
  import { conversationsStore } from "$lib/stores/conversations.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import type { ConversationsUpdatedEvent } from "$lib/types";

  let { children } = $props();

  // Track the unlisten function for cleanup
  let unlisten: UnlistenFn | null = null;

  // Track whether initial load is complete (to avoid toast for initial scan)
  let initialLoadComplete = $state(false);

  onMount(async () => {
    // 1. Set up event listener FIRST (before loading data)
    // This ensures we catch the conversations-updated event from the initial scan
    try {
      unlisten = await listenToConversationsUpdated(async (event: ConversationsUpdatedEvent) => {
        try {
          // Reload conversations while preserving selection
          await conversationsStore.reload();

          // Show notification only AFTER initial load is done
          // This prevents showing a toast for the initial scan
          if (initialLoadComplete) {
            const { newCount, updatedCount } = event;
            if (newCount > 0 || updatedCount > 0) {
              const parts: string[] = [];
              if (newCount > 0) {
                parts.push(`${newCount} new`);
              }
              if (updatedCount > 0) {
                parts.push(`${updatedCount} updated`);
              }
              toast.info(`Conversations: ${parts.join(", ")}`, 2000);
            }
          }

          // Mark initial load as complete after first successful reload
          initialLoadComplete = true;
        } catch (error) {
          console.error("[layout] Failed to reload conversations:", error);
          toast.error("Failed to update conversations");
        }
      });
    } catch (error) {
      console.error("[layout] Failed to set up event listener:", error);
    }

    // 2. Now load initial data (may be empty if backend scan is still in progress)
    try {
      await conversationsStore.load();

      // If we got data immediately, mark initial load as complete
      if (conversationsStore.conversations.length > 0) {
        initialLoadComplete = true;
      }

      await conversationsStore.restoreSelection();
    } catch (error) {
      console.error("[layout] Failed to load initial conversations:", error);
      toast.error("Failed to load conversations");
    }
  });

  // Cleanup on unmount (separate from onMount due to async limitations)
  onDestroy(() => {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  });
</script>

{@render children()}
