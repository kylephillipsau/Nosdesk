<!-- CreateTicketView.vue -->
<script setup lang="ts">
import { ref, computed, onBeforeUnmount, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import UserSelection from '@/components/ticketComponents/UserSelection.vue';
import CustomDropdown from '@/components/ticketComponents/CustomDropdown.vue';
import DeviceDetails from '@/components/ticketComponents/DeviceDetails.vue';
import TicketArticleBody from '@/components/ticketComponents/TicketArticleBody.vue';
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import users from '@/assets/users.json';
import TicketTitle from '@/components/ticketComponents/TicketTitle.vue';

interface Ticket {
  id?: number;
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
  articleContent?: string;
}

const router = useRouter();

const newTicket = ref<Ticket>({
  title: '',
  status: 'open' as TicketStatus,
  priority: 'low' as TicketPriority,
  created: new Date().toISOString(),
  modified: new Date().toISOString(),
  assignee: '',
  requester: '',
  device: undefined,
  linkedTicket: undefined,
  project: '',
  articleContent: '',
});

const selectedStatus = ref<TicketStatus>('open');
const selectedPriority = ref<TicketPriority>('low');
const showDeviceDetails = ref(false);
const deviceDetails = ref({
  id: '',
  name: '',
  hostname: '',
  serialNumber: '',
  model: '',
  warrantyStatus: '',
});

const usersFromJson = computed(() => users.users);

const updateStatus = (newStatus: string) => {
  selectedStatus.value = newStatus as TicketStatus;
  newTicket.value.status = newStatus as TicketStatus;
};

const updatePriority = (newPriority: string) => {
  selectedPriority.value = newPriority as TicketPriority;
  newTicket.value.priority = newPriority as TicketPriority;
};

const updateRequester = (userId: string) => {
  newTicket.value.requester = userId;
};

const updateAssignee = (userId: string) => {
  newTicket.value.assignee = userId;
};

const addDevice = () => {
  deviceDetails.value = { id: 'new-id', name: 'New Device', hostname: '', serialNumber: '', model: '', warrantyStatus: '' };
  showDeviceDetails.value = true;
  newTicket.value.device = deviceDetails.value;
};

const saveTicket = async () => {
  try {
    console.log('Saving ticket:', newTicket.value);
    // Reset unsaved changes flag
    hasUnsavedChanges.value = false;
    // @ts-ignore - Property added for router navigation guard
    window.hasUnsavedChanges = false;
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
    device: undefined,
    linkedTicket: undefined,
    project: '',
    articleContent: '',
  };
  selectedStatus.value = 'open';
  selectedPriority.value = 'low';
  showDeviceDetails.value = false;
  deviceDetails.value = {
    id: '',
    name: '',
    hostname: '',
    serialNumber: '',
    model: '',
    warrantyStatus: '',
  };
};

const updateTicketTitle = (newTitle: string) => {
  newTicket.value.title = newTitle;
};

const hasUnsavedChanges = ref(false);
const initialTicketState = ref('');

// Track form changes
const updateFormState = () => {
  const currentState = JSON.stringify(newTicket.value);
  hasUnsavedChanges.value = currentState !== initialTicketState.value;
  // @ts-ignore - Property added for router navigation guard
  window.hasUnsavedChanges = hasUnsavedChanges.value;
};

// Watch for unsaved changes when navigating
const handleBeforeUnload = (e: BeforeUnloadEvent) => {
  if (hasUnsavedChanges.value) {
    e.preventDefault();
    e.returnValue = '';
  }
};

// Initialize form tracking
onMounted(() => {
  initialTicketState.value = JSON.stringify(newTicket.value);
  window.addEventListener('beforeunload', handleBeforeUnload);
});

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload);
  // @ts-ignore - Property added for router navigation guard
  window.hasUnsavedChanges = false;
});

// Update form state whenever ticket data changes
watch(
  () => ({ ...newTicket.value }),
  () => updateFormState(),
  { deep: true }
);
</script>

<template>
  <div class="flex-1">
    <div class="flex flex-col">
      <div class="flex flex-col gap-4 p-6 mx-auto w-full max-w-7xl">

        <div class="flex items-center justify-between gap-4">
          <div class="flex-1">
            <TicketTitle :ticket-id="80085" :initial-title="newTicket.title" @update-title="updateTicketTitle" />
          </div>
          <button
            @click="saveTicket"
            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            :disabled="!newTicket.title || !newTicket.requester"
          >
            <span class="material-icons text-xl">Save Ticket</span>
          </button>
        </div>

        <!-- Navigation Warning -->
        <div v-if="hasUnsavedChanges" class="bg-yellow-500/10 border border-yellow-500/50 text-yellow-200 px-4 py-3 rounded-md mb-4 flex items-center gap-3">
          <span class="material-icons text-yellow-500">warning</span>
          <p>You have unsaved changes. Please save your work before leaving this page.</p>
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
            </div>

            <!-- Device Section -->
            <div v-if="!newTicket.device">
              <a href="#" @click.prevent="addDevice" class="block mb-2 text-blue-500 hover:underline">+ Add device</a>
            </div>
            <DeviceDetails
              v-if="showDeviceDetails"
              v-model:deviceId="deviceDetails.id"
              v-model:deviceName="deviceDetails.name"
              v-model:hostname="deviceDetails.hostname"
              v-model:serialNumber="deviceDetails.serialNumber"
              v-model:model="deviceDetails.model"
              v-model:warrantyStatus="deviceDetails.warrantyStatus"
            />

            <!-- Linked Ticket and Project -->
            <div class="flex flex-col gap-4">
              <div class="flex flex-col gap-1">
                <label class="text-sm text-slate-400">Linked Ticket (optional)</label>
                <input
                  v-model="newTicket.linkedTicket"
                  type="number"
                  placeholder="Enter linked ticket ID"
                  class="px-2 py-1 text-sm rounded bg-slate-700 text-slate-200 w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div class="flex flex-col gap-1">
                <label class="text-sm text-slate-400">Project (optional)</label>
                <input
                  v-model="newTicket.project"
                  type="text"
                  placeholder="Enter project name"
                  class="px-2 py-1 text-sm rounded bg-slate-700 text-slate-200 w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>
          </div>

          <!-- Article Body -->
          <TicketArticleBody v-model:content="newTicket.articleContent" :is-editing="true" />
        </div>
      </div>
    </div>
  </div>
</template>