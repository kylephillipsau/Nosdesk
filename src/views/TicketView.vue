<!-- TicketView.vue -->
<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useRecentTicketsStore } from "@/stores/recentTickets";
import { usePageTitle } from '@/composables/usePageTitle';
import TicketArticleBody from '@/components/ticketComponents/TicketArticleBody.vue';
import TicketDetails from '@/components/ticketComponents/TicketDetails.vue'
import DeviceDetails from '@/components/ticketComponents/DeviceDetails.vue';
import DeviceModal from '@/components/ticketComponents/DeviceModal.vue';
import CommentsAndAttachments from "@/components/ticketComponents/CommentsAndAttachments.vue";
import TicketTitle from "@/components/ticketComponents/TicketTitle.vue";
import LinkedTicketModal from "@/components/ticketComponents/LinkedTicketModal.vue";
import LinkedTicketPreview from "@/components/ticketComponents/LinkedTicketPreview.vue";
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import type { Ticket, Device } from '@/types/ticket';

interface NoteOrComment {
  id: number;
  content: string;
  author: string;
  createdAt: string;
  attachments?: { url: string; name: string }[];
}

const route = useRoute();
const router = useRouter();
const ticket = ref<Ticket | null>(null);
const recentTicketsStore = useRecentTicketsStore();
const { setCustomTitle } = usePageTitle();

const selectedStatus = ref<TicketStatus>("open");
const selectedPriority = ref<TicketPriority>("low");
const showDeviceModal = ref(false);

const fetchTicket = async (ticketId: string | string[]) => {
  const id = Number(ticketId);
  const ticketData = (await import("@/assets/tickets.json")).default;
  const foundTicket = ticketData.tickets.find((t: any) => t.id === id) as {
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

  if (!foundTicket) {
    router.push("/404");
    return;
  }

  ticket.value = {
    ...foundTicket,
    linkedTickets: foundTicket.linkedTickets || [],
    status: foundTicket.status as TicketStatus,
    priority: foundTicket.priority as TicketPriority,
    devices: foundTicket.devices || []
  };

  selectedStatus.value = ticket.value.status;
  selectedPriority.value = ticket.value.priority;

  const fromRecent = route.query.fromRecent === "true";
  recentTicketsStore.addRecentTicket(
    {
      id: ticket.value.id,
      title: ticket.value.title,
      status: ticket.value.status,
    },
    fromRecent,
  );
};

const formattedDate = (dateString: string | undefined) => {
  if (!dateString) return "";
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

const formattedCreatedDate = computed(() => formattedDate(ticket.value?.created));
const formattedModifiedDate = computed(() => formattedDate(ticket.value?.modified));

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
    // API endpoint needed: PATCH /api/tickets/{id}/status
  }
};

const updatePriority = (newPriority: TicketPriority) => {
  selectedPriority.value = newPriority;
  if (ticket.value) {
    ticket.value.priority = newPriority;
    // API endpoint needed: PATCH /api/tickets/{id}/priority
  }
};

const emit = defineEmits<{
  (e: 'update:ticket', ticket: { id: number; title: string } | null): void;
}>();

// Watch ticket changes to emit updates
watch(ticket, (newTicket) => {
  if (newTicket) {
    emit('update:ticket', {
      id: newTicket.id,
      title: newTicket.title
    });
    setCustomTitle(`#${newTicket.id} ${newTicket.title}`);
  } else {
    emit('update:ticket', null);
  }
}, { immediate: true });

const updateTicketTitle = (newTitle: string) => {
  if (ticket.value) {
    ticket.value.title = newTitle;
    setCustomTitle(`#${ticket.value.id} ${newTitle}`);
    // API endpoint needed: PATCH /api/tickets/{id}/title
  }
};

const handleAddDevice = (device: Device) => {
  if (ticket.value) {
    if (!ticket.value.devices) {
      ticket.value.devices = [];
    }
    ticket.value.devices.push(device);
    showDeviceModal.value = false;
    // API endpoint needed: PATCH /api/tickets/{id}/devices
  }
};

const removeDevice = (deviceId: string) => {
  if (ticket.value) {
    ticket.value.devices = ticket.value.devices?.filter((device) => device.id !== deviceId) || [];
    // API endpoint needed: DELETE /api/tickets/{id}/devices/{deviceId}
  }
};

const showLinkedTicketModal = ref(false);

const handleLinkTicket = (linkedTicketId: number) => {
  if (ticket.value) {
    if (!ticket.value.linkedTickets) {
      ticket.value.linkedTickets = [];
    }
    if (!ticket.value.linkedTickets.includes(linkedTicketId)) {
      ticket.value.linkedTickets.push(linkedTicketId);
    }
    // API endpoint needed: POST /api/tickets/{id}/links/{linkedTicketId}
  }
};

const unlinkTicket = (linkedTicketId: number) => {
  if (ticket.value) {
    ticket.value.linkedTickets = ticket.value.linkedTickets?.filter((id) => id !== linkedTicketId) || [];
    // API endpoint needed: DELETE /api/tickets/{id}/links/{linkedTicketId}
  }
};
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="flex flex-col">
      <div class="flex flex-col gap-4 p-6 mx-auto w-full max-w-8xl">
        <!-- Grid Container -->
        <div class="grid-container">
          <!-- Details (TicketDetails and DeviceDetails) -->
          <div class="details-area flex flex-col gap-4">
            <TicketDetails 
              :ticket="ticket" 
              :created-date="formattedCreatedDate" 
              :modified-date="formattedModifiedDate"
              :selected-status.sync="selectedStatus" 
              :selected-priority.sync="selectedPriority"
              :status-options="STATUS_OPTIONS" 
              :priority-options="PRIORITY_OPTIONS"
              @update:selectedStatus="updateStatus" 
              @update:selectedPriority="updatePriority" 
            />
            
            <!-- Device section -->
            <template v-if="ticket">
              <div v-if="ticket.devices?.length" class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Devices</h3>
                  <a 
                    href="#" 
                    @click.prevent="showDeviceModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    + Add device
                  </a>
                </div>
                <div class="flex flex-col gap-2">
                  <DeviceDetails
                    v-for="device in ticket.devices"
                    :key="device.id"
                    v-bind="device"
                    @remove="() => removeDevice(device.id)"
                  />
                </div>
              </div>
              <div v-else>
                <a href="#" @click.prevent="showDeviceModal = true" class="block text-blue-500 hover:underline">+ Add device</a>
              </div>

              <!-- Add DeviceModal component -->
              <DeviceModal
                :show="showDeviceModal"
                @close="showDeviceModal = false"
                @add-device="handleAddDevice"
              />

              <!-- Linked ticket section -->
              <div v-if="ticket.linkedTickets?.length" class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Linked Tickets</h3>
                  <a 
                    href="#" 
                    @click.prevent="showLinkedTicketModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    + Add linked ticket
                  </a>
                </div>
                <div class="flex flex-col gap-2">
                  <LinkedTicketPreview
                    v-for="linkedId in ticket.linkedTickets"
                    :key="linkedId"
                    :linked-ticket-id="linkedId"
                    @unlink="() => unlinkTicket(linkedId)"
                  />
                </div>
              </div>
              <div v-else>
                <a href="#" @click.prevent="showLinkedTicketModal = true" class="block text-blue-500 hover:underline">+ Add linked ticket</a>
              </div>

              <!-- Project section -->
              <div v-if="!ticket.project">
                <a href="#" class="block text-blue-500 hover:underline">+ Add to project</a>
              </div>
            </template>
          </div>

          <!-- TicketArticleBody -->
          <div class="article-area">
            <TicketArticleBody :initial-content="ticket.articleContent" />
          </div>

          <!-- Comments and Attachments -->
          <div class="comments-area">
            <CommentsAndAttachments :notes="ticket?.notesAndComments || []" />
          </div>
        </div>
      </div>
    </div>

    <div v-else class="p-6 text-center text-gray-400">
      Loading ticket...
    </div>

    <LinkedTicketModal 
      v-if="ticket"
      :show="showLinkedTicketModal"
      :current-ticket-id="ticket.id"
      :existing-linked-tickets="ticket.linkedTickets"
      @close="showLinkedTicketModal = false"
      @select-ticket="handleLinkTicket"
    />
  </div>
</template>

<style scoped>
.grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-template-areas: "details" "article" "comments";
  gap: 1rem;

  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas:
      "details article"
      "comments article";
  }

  @media (min-width: 1536px) {
    grid-template-columns: repeat(3, 1fr);
    grid-template-areas: "details article comments";
  }
}

.details-area {
  grid-area: details;
}

.article-area {
  grid-area: article;
}

.comments-area {
  grid-area: comments;
}
</style>