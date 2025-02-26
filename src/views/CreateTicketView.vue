<!-- CreateTicketView.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { useRouter } from 'vue-router';
import UserSelection from '@/components/ticketComponents/UserSelection.vue';
import CustomDropdown from '@/components/ticketComponents/CustomDropdown.vue';
import DeviceDetails from '@/components/ticketComponents/DeviceDetails.vue';
import DeviceModal from '@/components/ticketComponents/DeviceModal.vue';
import LinkedTicketModal from '@/components/ticketComponents/LinkedTicketModal.vue';
import LinkedTicketPreview from '@/components/ticketComponents/LinkedTicketPreview.vue';
import ProjectSelectionModal from '@/components/ticketComponents/ProjectSelectionModal.vue';
import TicketArticleBody from '@/components/ticketComponents/TicketArticleBody.vue';
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import type { Device } from '@/types/ticket';
import type { Project } from '@/types/project';
import users from '@/data/users.json';
import HeaderTitle from '@/components/HeaderTitle.vue';
import { useRecentTicketsStore } from '@/stores/recentTickets';

interface Ticket {
  id?: number;
  title: string;
  status: TicketStatus;
  priority: TicketPriority;
  created: string;
  modified: string;
  assignee: string;
  requester: string;
  devices?: Device[];
  linkedTickets?: number[];
  project?: string;
  articleContent?: string;
}

const router = useRouter();
const recentTicketsStore = useRecentTicketsStore();

// Generate a ticket number based on the most recent ticket
const generateTicketNumber = () => {
  // Get the highest ticket ID from recent tickets, or start at 10000 if none exist
  const recentTickets = recentTicketsStore.recentTickets;
  if (recentTickets.length === 0) {
    return 10000;
  }
  
  // Find the highest ticket ID
  const highestId = Math.max(...recentTickets.map(ticket => ticket.id));
  return highestId + 1;
};

// Generate a temporary ID for the draft ticket
const tempTicketId = ref(generateTicketNumber());

const newTicket = ref<Ticket>({
  title: '',
  status: 'open' as TicketStatus,
  priority: 'low' as TicketPriority,
  created: new Date().toISOString(),
  modified: new Date().toISOString(),
  assignee: '',
  requester: '',
  devices: [],
  linkedTickets: [],
  project: undefined,
  articleContent: '',
});

const selectedStatus = ref<TicketStatus>('open');
const selectedPriority = ref<TicketPriority>('low');
const showDeviceModal = ref(false);
const showLinkedTicketModal = ref(false);
const showProjectModal = ref(false);
const projectDetails = ref<Project | null>(null);

const usersFromJson = computed(() => users.users);

const updateStatus = (newStatus: string) => {
  selectedStatus.value = newStatus as TicketStatus;
  newTicket.value.status = newStatus as TicketStatus;
  updateDraftTicket();
};

const updatePriority = (newPriority: string) => {
  selectedPriority.value = newPriority as TicketPriority;
  newTicket.value.priority = newPriority as TicketPriority;
  updateDraftTicket();
};

const updateRequester = (userId: string) => {
  newTicket.value.requester = userId;
  updateDraftTicket();
};

const updateAssignee = (userId: string) => {
  newTicket.value.assignee = userId;
  updateDraftTicket();
};

const handleAddDevice = (device: Device) => {
  if (!newTicket.value.devices) {
    newTicket.value.devices = [];
  }
  newTicket.value.devices.push(device);
  showDeviceModal.value = false;
  updateDraftTicket();
};

const removeDevice = (deviceId: string) => {
  if (newTicket.value.devices) {
    newTicket.value.devices = newTicket.value.devices.filter(device => device.id !== deviceId);
    updateDraftTicket();
  }
};

const handleLinkTicket = (linkedTicketId: number) => {
  if (!newTicket.value.linkedTickets) {
    newTicket.value.linkedTickets = [];
  }
  if (!newTicket.value.linkedTickets.includes(linkedTicketId)) {
    newTicket.value.linkedTickets.push(linkedTicketId);
    updateDraftTicket();
  }
};

const unlinkTicket = (linkedTicketId: number) => {
  if (newTicket.value.linkedTickets) {
    newTicket.value.linkedTickets = newTicket.value.linkedTickets.filter(id => id !== linkedTicketId);
    updateDraftTicket();
  }
};

const handleAddToProject = async (projectId: number) => {
  // TODO: Replace with actual API call
  const mockProject: Project = {
    id: projectId,
    name: "Website Redesign",
    description: "Complete overhaul of the company website",
    status: 'active',
    ticketCount: 5
  }
  
  projectDetails.value = mockProject;
  newTicket.value.project = String(projectId);
  showProjectModal.value = false;
  updateDraftTicket();
};

const removeFromProject = () => {
  newTicket.value.project = undefined;
  projectDetails.value = null;
  updateDraftTicket();
};

// Function to update the draft ticket in the recent tickets list
const updateDraftTicket = () => {
  if (newTicket.value.title) {
    recentTicketsStore.addRecentTicket({
      id: tempTicketId.value,
      title: newTicket.value.title,
      status: newTicket.value.status,
      requester: newTicket.value.requester,
      assignee: newTicket.value.assignee,
      created: newTicket.value.created
    }, false, true); // Mark as draft
  }
};

const saveTicket = async () => {
  try {
    console.log('Saving ticket:', newTicket.value);
    
    // Update the ticket in recent tickets list as a non-draft
    if (newTicket.value.title) {
      recentTicketsStore.updateDraftStatus(tempTicketId.value, false);
    }
    
    // For now, just go back to the tickets list
    router.push('/tickets');
  } catch (error) {
    console.error('Error saving ticket:', error);
  }
};

const resetForm = () => {
  newTicket.value = {
    title: '',
    status: 'open' as TicketStatus,
    priority: 'low' as TicketPriority,
    created: new Date().toISOString(),
    modified: new Date().toISOString(),
    assignee: '',
    requester: '',
    devices: [],
    linkedTickets: [],
    project: undefined,
    articleContent: '',
  };
  selectedStatus.value = 'open';
  selectedPriority.value = 'low';
  showDeviceModal.value = false;
  showLinkedTicketModal.value = false;
  showProjectModal.value = false;
  projectDetails.value = null;
  
  // Remove from recent tickets if it exists
  recentTicketsStore.removeRecentTicket(tempTicketId.value);
};

const updateTicketTitle = (newTitle: string) => {
  newTicket.value.title = newTitle;
  updateDraftTicket();
};

// Initialize form tracking
onMounted(() => {
  // Set up auto-save for the draft ticket
  const autoSaveInterval = setInterval(() => {
    if (newTicket.value.title) {
      updateDraftTicket();
    }
  }, 30000); // Auto-save every 30 seconds
  
  // Clean up interval on component unmount
  onBeforeUnmount(() => {
    clearInterval(autoSaveInterval);
  });
});

// Update form state whenever ticket data changes
watch(
  () => ({ ...newTicket.value }),
  () => {
    // Auto-save draft when ticket data changes
    if (newTicket.value.title) {
      updateDraftTicket();
    }
  },
  { deep: true }
);

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
</script>

<template>
  <div class="flex-1">
    <div class="flex flex-col">
      <div class="flex flex-col gap-4 p-6 mx-auto w-full max-w-7xl">
        <div class="flex items-center justify-between gap-4">
          <div class="flex-1">
            <HeaderTitle 
              :identifier="tempTicketId" 
              :initial-title="newTicket.title" 
              :placeholder-text="'Enter ticket title...'"
              @update-title="updateTicketTitle" 
            />
          </div>
          <button
            @click="saveTicket"
            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            :disabled="!newTicket.title || !newTicket.requester"
          >
            Save Ticket
          </button>
        </div>

        <!-- Go Back Button -->
        <button
          @click="router.back()"
          class="mb-6 flex items-center gap-2 text-gray-400 hover:text-white transition-colors print:hidden"
        >
          <span>←</span> Go back
        </button>

        <!-- Grid Container -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="flex flex-col gap-4">
            <!-- Ticket Details Form -->
            <div class="flex flex-col bg-slate-800 rounded-2xl p-6 gap-4 shadow-lg">
              <h2 class="text-lg font-medium text-slate-100">Ticket Details</h2>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="flex flex-col gap-1">
                  <label class="text-sm text-slate-400">Requester</label>
                  <UserSelection
                    v-model="newTicket.requester"
                    :users="usersFromJson"
                    placeholder="Search or select Requester..."
                    type="requester"
                    class="w-full"
                  />
                </div>

                <div class="flex flex-col gap-1">
                  <label class="text-sm text-slate-400">Assignee</label>
                  <UserSelection
                    v-model="newTicket.assignee"
                    :users="usersFromJson"
                    placeholder="Search or select Assignee..."
                    type="assignee"
                    class="w-full"
                  />
                </div>

                <div class="flex flex-col gap-1">
                  <label class="text-sm text-slate-400">Status</label>
                  <CustomDropdown
                    :value="selectedStatus"
                    :options="STATUS_OPTIONS"
                    type="status"
                    @update:value="updateStatus"
                    class="w-full"
                  />
                </div>

                <div class="flex flex-col gap-1">
                  <label class="text-sm text-slate-400">Priority</label>
                  <CustomDropdown
                    :value="selectedPriority"
                    :options="PRIORITY_OPTIONS"
                    type="priority"
                    @update:value="updatePriority"
                    class="w-full"
                  />
                </div>
              </div>

              <!-- Devices Section -->
              <div class="flex flex-col gap-2">
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
                <div v-if="newTicket.devices?.length" class="flex flex-col gap-2">
                  <DeviceDetails
                    v-for="device in newTicket.devices"
                    :key="device.id"
                    v-bind="device"
                    @remove="() => removeDevice(device.id)"
                  />
                </div>
              </div>

              <!-- Linked Tickets Section -->
              <div class="flex flex-col gap-2">
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
                <div v-if="newTicket.linkedTickets?.length" class="flex flex-col gap-2">
                  <LinkedTicketPreview
                    v-for="linkedId in newTicket.linkedTickets"
                    :key="linkedId"
                    :linked-ticket-id="linkedId"
                    @unlink="() => unlinkTicket(linkedId)"
                  />
                </div>
              </div>

              <!-- Project Section -->
              <div class="flex flex-col gap-2">
                <div v-if="newTicket.project" class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Project</h3>
                  <a 
                    href="#" 
                    @click.prevent="showProjectModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    Change project
                  </a>
                </div>

                <div v-if="newTicket.project && projectDetails" class="bg-slate-700 p-3 rounded-lg">
                  <!-- Project Info -->
                  <div class="flex flex-col gap-2">
                    <div class="flex items-start justify-between">
                      <div class="flex-1">
                        <h4 class="text-sm font-medium text-white">{{ projectDetails.name }}</h4>
                        <p class="text-sm text-slate-400 mt-0.5 line-clamp-2">{{ projectDetails.description }}</p>
                      </div>
                      <div class="flex items-start gap-2 ml-4">
                        <button
                          @click="removeFromProject"
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
                        #{{ newTicket.project }}
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
              </div>
            </div>
          </div>

          <div class="flex flex-col gap-4">
            <!-- Ticket Notes -->
            <TicketArticleBody v-model:content="newTicket.articleContent" />
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <DeviceModal
      :show="showDeviceModal"
      @close="showDeviceModal = false"
      @add-device="handleAddDevice"
    />

    <LinkedTicketModal 
      :show="showLinkedTicketModal"
      :current-ticket-id="tempTicketId"
      :existing-linked-tickets="newTicket.linkedTickets"
      @close="showLinkedTicketModal = false"
      @select-ticket="handleLinkTicket"
    />

    <ProjectSelectionModal
      :show="showProjectModal"
      :current-project-id="newTicket.project ? Number(newTicket.project) : undefined"
      @close="showProjectModal = false"
      @select-project="handleAddToProject"
    />
  </div>
</template>