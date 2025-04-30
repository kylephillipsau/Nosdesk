<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { useRouter } from "vue-router";
import type { TicketStatus, TicketPriority } from "@/constants/ticketOptions";
import StatusBadge from "@/components/StatusBadge.vue";
import UserAvatar from "@/components/UserAvatar.vue";
import ticketService from "@/services/ticketService";
import type { Ticket, Device } from "@/services/ticketService";

const props = defineProps<{
  linkedTicketId: number;
  currentTicketId?: number;
}>();

const emit = defineEmits<{
  (e: "unlink"): void;
  (e: "view"): void;
}>();

const router = useRouter();
const linkedTicket = ref<Ticket | null>(null);
const isNavigating = ref(false);

const isSameAsCurrentTicket = computed(() => {
  return props.currentTicketId && props.linkedTicketId === props.currentTicketId;
});

const fetchLinkedTicket = async () => {
  if (isSameAsCurrentTicket.value) {
    console.log(`Skipping fetch for ticket #${props.linkedTicketId} as it's the same as the current ticket #${props.currentTicketId}`);
    return;
  }
  
  try {
    console.log(`Fetching linked ticket #${props.linkedTicketId}`);
    const fetchedTicket = await ticketService.getTicketById(props.linkedTicketId);
    
    if (fetchedTicket) {
      linkedTicket.value = fetchedTicket;
      console.log(`Successfully fetched linked ticket #${props.linkedTicketId}:`, fetchedTicket);
    }
  } catch (error) {
    console.error(`Error fetching linked ticket #${props.linkedTicketId}:`, error);
  }
};

const viewTicket = async () => {
  emit("view");

  if (isNavigating.value || !props.linkedTicketId) return;

  try {
    isNavigating.value = true;
    await router.push(`/tickets/${props.linkedTicketId}`);
  } catch (error) {
    console.error("Navigation error:", error);
    isNavigating.value = false;
  }
};

onMounted(() => {
  if (isSameAsCurrentTicket.value) {
    console.log(`Skipping fetch for ticket #${props.linkedTicketId} as it's the same as the current ticket #${props.currentTicketId}`);
    return;
  }
  fetchLinkedTicket();
});

onBeforeUnmount(() => {
  linkedTicket.value = null;
  isNavigating.value = false;
});

const formattedDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
};
</script>

<template>
  <div v-if="linkedTicket && !isSameAsCurrentTicket" class="bg-slate-800 rounded-lg overflow-hidden">
    <!-- Header with status and actions -->
    <div class="py-2 px-4 bg-slate-700/50 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <StatusBadge
          type="status"
          :value="linkedTicket.status"
          custom-classes="w-1 h-2.5 flex-shrink-0"
        />
        <span class="text-gray-400 text-sm">#{{ linkedTicket.id }}</span>
        <div class="flex items-center gap-2">
          
          <h3 class="text-white font-medium flex-1">
            {{ linkedTicket.title }}
          </h3>
        </div>
      </div>
      <div class="flex items-center gap-2">
        
        <button
          @click="viewTicket"
          :disabled="isNavigating"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors disabled:opacity-50"
          title="View ticket"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path
              fill-rule="evenodd"
              d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
        <button
          @click="emit('unlink')"
          :disabled="isNavigating"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors disabled:opacity-50"
          title="Unlink ticket"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path
              fill-rule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- Ticket content -->
    <div class="flex flex-col p-4 gap-2">
      <!-- Title and ID -->

      <!-- Details grid -->
      <div class="grid grid-cols-2 gap-1 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Priority:</span>
          <StatusBadge type="priority" :value="linkedTicket.priority" short />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Created:</span>
          <span class="text-white">{{
            formattedDate(linkedTicket.created)
          }}</span>
        </div>
        <div class="flex items-center gap-2 col-span-2">
          <span class="text-gray-400 min-w-[70px]">Assignee:</span>
          <UserAvatar 
            v-if="linkedTicket.assignee" 
            :name="linkedTicket.assignee" 
            size="xs" 
            :showName="true"
          />
          <span v-else class="text-white">Unassigned</span>
        </div>
        <div class="flex items-center gap-2 col-span-2">
          <span class="text-gray-400 min-w-[70px]">Requester:</span>
          <UserAvatar 
            v-if="linkedTicket.requester" 
            :name="linkedTicket.requester" 
            size="xs" 
            :showName="true"
          />
          <span v-else class="text-white">None</span>
        </div>
      </div>
    </div>
  </div>
</template>
