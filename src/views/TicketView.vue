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
import LinkedTicketModal from "@/components/ticketComponents/LinkedTicketModal.vue";
import LinkedTicketPreview from "@/components/ticketComponents/LinkedTicketPreview.vue";
import ProjectSelectionModal from "@/components/ticketComponents/ProjectSelectionModal.vue";
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import type { Ticket as ImportedTicket, Device } from '@/types/ticket';
import type { Project } from '@/types/project';

interface NoteWithAttachments {
  id: number;
  content: string;
  author: string;
  createdAt: string;
  attachments?: { url: string; name: string }[];
}

interface LocalTicket extends ImportedTicket {
  commentsAndAttachments?: NoteWithAttachments[];
}

interface RawTicket {
  id: number;
  title: string;
  status: string;
  priority: string;
  created: string;
  modified: string;
  assignee: string;
  requester: string;
  device?: {
    hostname: string;
    serialNumber: string;
    model: string;
    warrantyStatus: string;
  };
  notesAndComments?: NoteWithAttachments[];
  commentsAndAttachments?: NoteWithAttachments[];
  linkedTickets?: number[];
  devices?: Device[];
  project?: string;
  articleContent?: string;
}

const route = useRoute();
const router = useRouter();
const ticket = ref<LocalTicket | null>(null);
const projectDetails = ref<Project | null>(null);
const recentTicketsStore = useRecentTicketsStore();
const { setCustomTitle } = usePageTitle();

const selectedStatus = ref<TicketStatus>("open");
const selectedPriority = ref<TicketPriority>("low");
const showDeviceModal = ref(false);
const showProjectModal = ref(false);

// Add back navigation functionality
const previousRoute = ref<string>('');

// Watch route changes to store previous route
watch(() => route.fullPath, (newPath, oldPath) => {
  if (oldPath) {
    previousRoute.value = oldPath;
  }
}, { immediate: true });

const handleBack = () => {
  if (previousRoute.value) {
    router.push(previousRoute.value);
  } else {
    router.push('/tickets'); // Default fallback to tickets list
  }
};

const fetchTicket = async (ticketId: string | string[]) => {
  const id = Number(ticketId);
  const ticketData = (await import("@/assets/tickets.json")).default;
  const foundTicket = ticketData.tickets.find((t) => t.id === id) as RawTicket;

  if (!foundTicket) {
    router.push("/404");
    return;
  }

  // Transform notesAndComments to commentsAndAttachments
  const commentsAndAttachments = foundTicket.notesAndComments || foundTicket.commentsAndAttachments || [];

  ticket.value = {
    ...foundTicket,
    linkedTickets: foundTicket.linkedTickets || [],
    status: foundTicket.status as TicketStatus,
    priority: foundTicket.priority as TicketPriority,
    devices: foundTicket.devices || (foundTicket.device ? [foundTicket.device] : []),
    commentsAndAttachments
  } as LocalTicket;

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

  if (foundTicket.project) {
    await fetchProjectDetails(foundTicket.project);
  }
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

const handleAddToProject = async (projectId: number) => {
  if (ticket.value) {
    // TODO: Replace with actual API call
    const mockProject: Project = {
      id: projectId,
      name: "Website Redesign",
      description: "Complete overhaul of the company website",
      status: 'active',
      ticketCount: 5
    }
    
    projectDetails.value = mockProject
    ticket.value.project = String(projectId)
    showProjectModal.value = false
    // TODO: Update ticket in backend
    console.log(`Added ticket ${ticket.value.id} to project ${projectId}`)
  }
}

// Add fetchProjectDetails function
const fetchProjectDetails = async (projectId: string) => {
  // TODO: Replace with actual API call
  const mockProject: Project = {
    id: Number(projectId),
    name: "Website Redesign",
    description: "Complete overhaul of the company website",
    status: 'active',
    ticketCount: 5
  }
  
  projectDetails.value = mockProject
}

// Add this function to handle project navigation
const viewProject = (projectId: string) => {
  router.push(`/projects/${projectId}`)
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'text-green-400'
    case 'completed':
      return 'text-blue-400'
    case 'archived':
      return 'text-gray-400'
    default:
      return 'text-slate-400'
  }
}

// Add save functionality
const isSaving = ref(false);

const handleSave = async () => {
  if (isSaving.value) return;
  
  try {
    isSaving.value = true;
    // TODO: Implement actual save logic here
    await new Promise(resolve => setTimeout(resolve, 500)); // Simulate API call
    console.log('Changes saved');
  } catch (error) {
    console.error('Error saving changes:', error);
  } finally {
    isSaving.value = false;
  }
};

// Add function to handle new comments
const handleAddComment = async (data: { content: string; attachments: { url: string; name: string }[] }) => {
  if (!ticket.value) return;

  const newComment: NoteWithAttachments = {
    id: Date.now(), // Temporary ID until we have a backend
    content: data.content,
    author: "Kyle Phillips", // Hardcoded for now, should come from auth
    createdAt: new Date().toISOString(),
    attachments: data.attachments
  };

  if (!ticket.value.commentsAndAttachments) {
    ticket.value.commentsAndAttachments = [];
  }

  ticket.value.commentsAndAttachments.push(newComment);
  // TODO: API call to save comment
  console.log('New comment added:', newComment);
};
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="flex flex-col">
      <!-- Navigation and actions bar -->
      <div class="pt-4 px-6 flex justify-between items-center">
        <button
          @click="handleBack"
          class="text-slate-400 hover:text-white text-sm flex items-center gap-1 group"
        >
          <span class="text-xs group-hover:-translate-x-0.5 transition-transform">←</span>
          Go back
        </button>
        
        <button
          @click="handleSave"
          :disabled="isSaving"
          class="px-3 py-1.5 bg-blue-500 text-white text-sm rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 transition-colors"
        >
          <span v-if="isSaving">Saving...</span>
          <span v-else>Save changes</span>
        </button>
      </div>
      
      <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
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
              <div class="flex flex-col gap-2">
                <div v-if="ticket.project" class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Project</h3>
                  <a 
                    href="#" 
                    @click.prevent="showProjectModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    Change project
                  </a>
                </div>

                <div v-if="ticket.project && projectDetails" class="bg-slate-700 p-3 rounded-lg">
                  <!-- Project Info -->
                  <div class="flex flex-col gap-2">
                    <div class="flex items-start justify-between">
                      <div class="flex-1">
                        <h4 class="text-sm font-medium text-white">{{ projectDetails.name }}</h4>
                        <p class="text-sm text-slate-400 mt-0.5 line-clamp-2">{{ projectDetails.description }}</p>
                      </div>
                      <div class="flex items-start gap-2 ml-4">
                        <button
                          @click="viewProject(ticket.project!)"
                          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
                          title="View project"
                        >
                          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                            <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
                          </svg>
                        </button>
                        <button
                          @click="ticket.project = undefined; projectDetails = null"
                          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
                          title="Remove from project"
                        >
                          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                          </svg>
                        </button>
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="text-xs px-2 py-0.5 bg-slate-600/50 text-slate-300 rounded">
                        #{{ ticket.project }}
                      </span>
                      <span :class="[getStatusColor(projectDetails.status), 'text-xs']">
                        {{ projectDetails.status }}
                      </span>
                      <span class="text-xs text-slate-400">
                        {{ projectDetails.ticketCount }} tickets
                      </span>
                    </div>
                  </div>
                </div>

                <div v-else>
                  <a 
                    href="#" 
                    @click.prevent="showProjectModal = true" 
                    class="block text-blue-500 hover:underline"
                  >
                    + Add to project
                  </a>
                </div>

                <!-- Project Selection Modal -->
                <ProjectSelectionModal
                  :show="showProjectModal"
                  :current-project-id="ticket.project ? Number(ticket.project) : undefined"
                  @close="showProjectModal = false"
                  @select-project="handleAddToProject"
                />
              </div>
            </template>
          </div>

          <!-- TicketArticleBody -->
          <div class="article-area">
            <TicketArticleBody :initial-content="ticket.articleContent" />
          </div>

          <!-- Comments and Attachments -->
          <div class="comments-area">
            <CommentsAndAttachments 
              :notes="ticket?.commentsAndAttachments || []"
              :current-user="ticket?.assignee || 'Unknown User'"
              @add-comment="handleAddComment"
            />
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