// TicketView.vue
<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useRecentTicketsStore } from "@/stores/recentTickets";
import StatusBadge from '@/components/StatusBadge.vue'
import TicketDetails from '@/components/TicketDetails.vue'
import NotesAndComments from "@/components/NotesAndComments.vue";
import PageHeader from '@/components/PageHeader.vue'
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions'
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions'

interface Ticket {
  id: number;
  title: string;
  status: TicketStatus;
  priority: TicketPriority;
  created: string;
  assignee: string;
  requester: string;
}

const route = useRoute();
const router = useRouter();
const ticket = ref<Ticket | null>(null);
const recentTicketsStore = useRecentTicketsStore();

const selectedStatus = ref<TicketStatus>("open");
const selectedPriority = ref<TicketPriority>("low");

const fetchTicket = async (ticketId: string | string[]) => {
  const id = Number(ticketId);
  const ticketData = (await import("@/assets/tickets.json")).default;
  const foundTicket = ticketData.tickets.find((t) => t.id === id);

  if (!foundTicket) {
    router.push("/404");
    return;
  }

  const typedTicket: Ticket = {
    id: foundTicket.id,
    title: foundTicket.title,
    status: foundTicket.status as TicketStatus,
    priority: foundTicket.priority as TicketPriority,
    created: foundTicket.created,
    assignee: foundTicket.assignee,
    requester: foundTicket.requester
  };

  ticket.value = typedTicket;
  selectedStatus.value = typedTicket.status;
  selectedPriority.value = typedTicket.priority;

  const fromRecent = route.query.fromRecent === "true";
  recentTicketsStore.addRecentTicket(
    {
      id: typedTicket.id,
      title: typedTicket.title,
      status: typedTicket.status,
    },
    fromRecent,
  );
};

const formattedDate = computed(() => {
  if (!ticket.value?.created) return "";
  const date = new Date(ticket.value.created);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
});

watch(
  () => route.params.id,
  (newId) => {
    if (newId) {
      fetchTicket(newId);
    }
  },
  { immediate: true },
);

const updateStatus = (newStatus: TicketStatus) => {
  selectedStatus.value = newStatus;
  if (ticket.value) {
    ticket.value.status = newStatus;
    // Add API call here to update status in backend
  }
};

const updatePriority = (newPriority: TicketPriority) => {
  selectedPriority.value = newPriority;
  if (ticket.value) {
    ticket.value.priority = newPriority;
    // Add API call here to update priority in backend
  }
};
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="flex flex-col min-h-full">
      <PageHeader :title="`${ticket.title} #${ticket.id}`">
        <template #actions>
          <div class="flex items-center gap-3">
            <StatusBadge type="status" :value="ticket.status" />
            <StatusBadge type="priority" :value="ticket.priority" />
          </div>
        </template>
      </PageHeader>

      <div class="flex flex-col gap-4 p-6 mx-auto w-full max-w-7xl">
        <!-- Go Back Button -->
        <button @click="router.back()"
          class="mb-6 flex items-center gap-2 text-gray-400 hover:text-white transition-colors print:hidden">
          <span>←</span> Go back
        </button>

        <!-- Grid Container -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <!-- Ticket Details Column -->
          <TicketDetails :ticket="ticket" :formatted-date="formattedDate" :selected-status.sync="selectedStatus"
            :selected-priority.sync="selectedPriority" :status-options="STATUS_OPTIONS"
            :priority-options="PRIORITY_OPTIONS" @update:selectedStatus="updateStatus"
            @update:selectedPriority="updatePriority" />


          <!-- Notes and Comments Column -->
          <NotesAndComments />
        </div>
      </div>
    </div>

    <div v-else class="p-6 text-center text-gray-400">
      Loading ticket...
    </div>
  </div>
</template>