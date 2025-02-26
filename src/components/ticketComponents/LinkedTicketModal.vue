# views/components/ticketComponents/LinkedTicketModal.vue
<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import type { Ticket } from '@/types/ticket';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import StatusBadge from '@/components/StatusBadge.vue';
import Modal from '@/components/Modal.vue';

const router = useRouter();
const props = defineProps<{
  show: boolean;
  currentTicketId: number;
  existingLinkedTickets?: number[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select-ticket', ticketId: number): void;
}>();

const searchQuery = ref('');
const tickets = ref<Ticket[]>([]);
const filteredTickets = ref<Ticket[]>([]);

// Load tickets from the JSON file
const loadTickets = async () => {
  const ticketData = (await import("@/data/tickets.json")).default;
  // Filter out the current ticket and any already linked tickets
  tickets.value = ticketData.tickets.map((t: any) => ({
    ...t,
    status: t.status as TicketStatus,
    priority: t.priority as TicketPriority
  })).filter((t: Ticket) => 
    t.id !== props.currentTicketId && 
    !props.existingLinkedTickets?.includes(t.id)
  );
  filterTickets();
};

// Filter tickets based on search query
const filterTickets = () => {
  if (!searchQuery.value.trim()) {
    filteredTickets.value = tickets.value;
    return;
  }
  
  const query = searchQuery.value.toLowerCase();
  filteredTickets.value = tickets.value.filter((ticket: Ticket) => 
    ticket.title.toLowerCase().includes(query) || 
    String(ticket.id).includes(query)
  );
};

// Watch for search query changes
watch(searchQuery, () => {
  filterTickets();
});

// Watch for modal visibility
watch(() => props.show, (newValue) => {
  if (newValue) {
    loadTickets();
    searchQuery.value = '';
  }
});

const selectTicket = (ticketId: number) => {
  emit('select-ticket', ticketId);
  emit('close');
};

const viewTicket = (ticketId: number, event: Event) => {
  event.stopPropagation(); // Prevent triggering the link action
  router.push(`/tickets/${ticketId}`);
};
</script>

<template>
  <Modal :show="show" title="Link to Another Ticket" @close="emit('close')">
    <div class="flex flex-col gap-2">
      <p class="text-sm text-gray-400">
        Select a ticket to create a link. Click the view button to open the ticket in a new tab.
      </p>

      <!-- Search -->
      <div>
        <input type="text" 
          v-model="searchQuery"
          class="w-full p-2 rounded-lg border-gray-600 bg-slate-800 text-white placeholder-gray-400 focus:border-blue-500 focus:ring-blue-500"
          placeholder="Search tickets by ID or title..."
        >
      </div>

      <!-- Tickets list -->
      <div class="max-h-96 overflow-y-auto">
        <div v-if="filteredTickets.length === 0" class="text-center py-4 text-gray-400">
          No tickets found
        </div>
        <div v-else class="space-y-1">
          <div v-for="ticket in filteredTickets" :key="ticket.id"
            class="group px-2 py-1.5 rounded-lg transition-colors duration-200 hover:bg-slate-700 relative cursor-pointer flex items-center justify-between"
            @click="selectTicket(ticket.id)">
            <!-- Left side content -->
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <StatusBadge 
                type="status" 
                :value="ticket.status"
                custom-classes="w-2.5 h-2.5 rounded-full"
              />
              <span class="text-sm text-gray-400">#{{ ticket.id }}</span>
              <span 
                class="text-sm text-white truncate relative"
                :title="ticket.title"
              >
                {{ ticket.title }}
              </span>
            </div>

            <!-- Right side content -->
            <div class="flex items-center gap-4">
              <button 
                @click="viewTicket(ticket.id, $event)"
                class="text-gray-400 hover:text-white text-sm opacity-0 group-hover:opacity-100 transition-all">
                View
              </button>
              <button 
                class="text-blue-400 hover:text-blue-300 text-sm">
                Link
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-6 flex justify-end">
      <button type="button"
        class="px-4 py-2 text-sm text-slate-300 hover:text-slate-100"
        @click="emit('close')">
        Cancel
      </button>
    </div>
  </Modal>
</template>
