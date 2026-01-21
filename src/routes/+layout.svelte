<script lang="ts">
  import "../app.css";
  import { listenToConversationsUpdated, type UnlistenFn } from "$lib/services/tauri";
  import { conversationsStore } from "$lib/stores/conversations.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import type { ConversationsUpdatedEvent } from "$lib/types";

  let { children } = $props();

  // Track the unlisten function for cleanup
  let unlisten: UnlistenFn | null = null;

  // Set up Tauri event listener for real-time conversation updates
  $effect(() => {
    // Set up the listener
    listenToConversationsUpdated(async (event: ConversationsUpdatedEvent) => {
      try {
        // Reload conversations while preserving selection
        await conversationsStore.reload();

        // Show subtle notification
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
      } catch (error) {
        console.error("[layout] Failed to reload conversations:", error);
        toast.error("Failed to update conversations");
      }
    }).then((fn) => {
      unlisten = fn;
    });

    // Cleanup function when component unmounts
    return () => {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    };
  });
</script>

{@render children()}
