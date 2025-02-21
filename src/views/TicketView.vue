<!-- TicketView.vue -->
<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useRecentTicketsStore } from "@/stores/recentTickets";
import TicketArticleBody from '@/components/ticketComponents/TicketArticleBody.vue';
import TicketDetails from '@/components/ticketComponents/TicketDetails.vue'
import DeviceDetails from '@/components/ticketComponents/DeviceDetails.vue';
import NotesAndComments from "@/components/ticketComponents/NotesAndComments.vue";
import TicketTitle from "@/components/ticketComponents/TicketTitle.vue";
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions'
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions'

interface Ticket {
  id: number;
  title: string;
  status: TicketStatus;
  priority: TicketPriority;
  created: string;
  modified: string;
  assignee: string;
  requester: string;
  device?: {
    id?: string;
    name?: string;
    hostname?: string;
    serialNumber?: string;
    model?: string;
    warrantyStatus?: string;
  };
  linkedTicket?: number;
  project?: string;
  notesAndComments?: NoteOrComment[];
  articleContent?: string;
}

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
    modified: foundTicket.modified,
    assignee: foundTicket.assignee,
    requester: foundTicket.requester,
    device: foundTicket.device,
    // linkedTicket: foundTicket.linkedTicket,
    // project: foundTicket.project,
    notesAndComments: foundTicket.notesAndComments,
    articleContent: foundTicket.articleContent
  };

  ticket.value = typedTicket;
  selectedStatus.value = typedTicket.status;
  selectedPriority.value = typedTicket.priority;

  if (typedTicket.device) {
    deviceDetails.value = {
      id: typedTicket.device.id || '',
      name: typedTicket.device.name || '',
      hostname: typedTicket.device.hostname || '',
      serialNumber: typedTicket.device.serialNumber || '',
      model: typedTicket.device.model || '',
      warrantyStatus: typedTicket.device.warrantyStatus || ''
    };
    showDeviceDetails.value = true;
  } else {
    showDeviceDetails.value = false;
  }

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

const updateTicketTitle = (newTitle: string) => {
  if (ticket.value) {
    ticket.value.title = newTitle;
  }
};

const showDeviceDetails = ref(false);
const deviceDetails = ref({
  id: '',
  name: '',
  hostname: '',
  serialNumber: '',
  model: '',
  warrantyStatus: ''
});
const addDevice = () => {
  // If device already exists, this might be an edit operation
  if (ticket.value?.device) {
    // Handle edit or show edit modal
  } else {
    // For adding a new device, you can keep your current logic
    deviceDetails.value = { id: 'new-id', name: 'New Device', hostname: '', serialNumber: '', model: '', warrantyStatus: '' };
    showDeviceDetails.value = true;
    // Here you would save this to backend
  }
};
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="flex flex-col">
      <div class="flex flex-col gap-4 p-6 mx-auto w-full max-w-8xl">

          <TicketTitle :ticket-id="ticket.id" :initial-title="ticket.title" @update-title="updateTicketTitle" />

        <!-- Grid Container -->
        <div class="grid-container">
          <!-- Details (TicketDetails and DeviceDetails) -->
          <div class="details-area flex flex-col gap-4">
            <TicketDetails :ticket="ticket" :created-date="formattedCreatedDate" :modified-date="formattedModifiedDate"
              :selected-status.sync="selectedStatus" :selected-priority.sync="selectedPriority"
              :status-options="STATUS_OPTIONS" :priority-options="PRIORITY_OPTIONS"
              @update:selectedStatus="updateStatus" @update:selectedPriority="updatePriority" />
            <div v-if="!ticket.device">
              <a href="#" @click.prevent="addDevice" class="block mb-2 text-blue-500 hover:underline">+ Add device</a>
            </div>

            <DeviceDetails v-if="showDeviceDetails" :deviceId="deviceDetails.id" :deviceName="deviceDetails.name"
              :hostname="deviceDetails.hostname" :serialNumber="deviceDetails.serialNumber" :model="deviceDetails.model"
              :warrantyStatus="deviceDetails.warrantyStatus" />

            <div v-if="!ticket.linkedTicket">
              <a href="#" class="block mb-2 text-blue-500 hover:underline">+ Add linked ticket</a>
            </div>
            <div v-if="!ticket.project">
              <a href="#" class="block text-blue-500 hover:underline">+ Add to project</a>
            </div>
          </div>

          <!-- TicketArticleBody -->
          <div class="article-area">
            <TicketArticleBody :initial-content="ticket.articleContent" />
          </div>

          <!-- NotesAndComments -->
          <div class="comments-area">
            <NotesAndComments :notesAndComments="ticket?.notesAndComments || []" />
          </div>
        </div>
      </div>
    </div>

    <div v-else class="p-6 text-center text-gray-400">
      Loading ticket...
    </div>
  </div>
</template>

<style scoped>
.grid-container {
  /* Base grid setup (small screens: 1 column, stacked vertically) */
  display: grid;
  grid-template-columns: 1fr;
  /* Single column */
  grid-template-rows: auto;
  /* Auto height for each row */
  grid-template-areas: "details" "article" "comments";
  /* Explicit stacking order */
  gap: 1rem;
  /* Equivalent to Tailwind's gap-4 */

  /* Medium to large screens (lg: 2 columns, NotesAndComments under Details in left column) */
  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas:
      "details article"
      "comments article";
  }

  /* Extra large screens (2xl: 3 columns) */
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