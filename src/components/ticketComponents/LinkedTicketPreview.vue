<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useRouter } from 'vue-router';
import type { Ticket, Device } from '@/types/ticket';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import StatusBadge from '@/components/StatusBadge.vue';

const props = defineProps<{
  linkedTicketId: number;
}>();

const emit = defineEmits<{
  (e: 'unlink'): void;
}>();

const router = useRouter();
const linkedTicket = ref<Ticket | null>(null);
const isNavigating = ref(false);

const fetchLinkedTicket = async () => {
  try {
    const ticketData = (await import("@/data/tickets.json")).default;
    const foundTicket = ticketData.tickets.find((t: any) => t.id === props.linkedTicketId) as {
      id: number;
      title: string;
      status: string;
      priority: string;
      created: string;
      modified: string;
      assignee: string;
      requester: string;
      devices?: Device[];
      linkedTickets?: number[];
      project?: string;
      notesAndComments?: {
        id: number;
        content: string;
        author: string;
        createdAt: string;
        attachments?: { url: string; name: string }[];
      }[];
      articleContent?: string;
    };
    
    if (foundTicket) {
      linkedTicket.value = {
        ...foundTicket,
        status: foundTicket.status as TicketStatus,
        priority: foundTicket.priority as TicketPriority,
        linkedTickets: foundTicket.linkedTickets || [],
        devices: foundTicket.devices || []
      };
    }
  } catch (error) {
    console.error('Error fetching linked ticket:', error);
  }
};

const viewTicket = async (event: Event) => {
  event.preventDefault();
  event.stopPropagation();
  
  if (isNavigating.value || !props.linkedTicketId) return;
  
  try {
    isNavigating.value = true;
    await router.push(`/tickets/${props.linkedTicketId}`);
  } catch (error) {
    console.error('Navigation error:', error);
    isNavigating.value = false;
  }
};

onMounted(() => {
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
    year: "numeric"
  });
};
</script>

<template>
  <div v-if="linkedTicket" class="bg-slate-800 rounded-lg overflow-hidden">
    <!-- Header with status and actions -->
    <div class="px-4 py-3 bg-slate-700/50 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-sm text-gray-400">Linked Ticket</span>
        <StatusBadge 
          type="status" 
          :value="linkedTicket.status"
        />
      </div>
      <button 
        @click="emit('unlink')"
        :disabled="isNavigating"
        class="text-red-400 hover:text-red-300 text-sm transition-colors disabled:opacity-50">
        Unlink
      </button>
    </div>

    <!-- Ticket content -->
    <div class="flex flex-col p-4 gap-2">
      <!-- Title and ID -->
      <div class="flex items-center gap-2 mb-3">
        <span class="text-gray-400 text-sm">#{{ linkedTicket.id }}</span>
        <h3 class="text-white font-medium flex-1">{{ linkedTicket.title }}</h3>
      </div>

      <!-- Details grid -->
      <div class="grid grid-cols-2 gap-1 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Priority:</span>
          <StatusBadge type="priority" :value="linkedTicket.priority" short />
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Created:</span>
          <span class="text-white">{{ formattedDate(linkedTicket.created) }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Assignee:</span>
          <span class="text-white">{{ linkedTicket.assignee }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-400">Requester:</span>
          <span class="text-white">{{ linkedTicket.requester }}</span>
        </div>
      </div>

      <!-- View button -->
      <button 
        @click="viewTicket"
        :disabled="isNavigating"
        class="mt-4 w-full px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded transition-colors text-sm disabled:opacity-50 disabled:cursor-not-allowed">
        {{ isNavigating ? 'Loading...' : 'View Ticket' }}
      </button>
    </div>
  </div>
</template> 