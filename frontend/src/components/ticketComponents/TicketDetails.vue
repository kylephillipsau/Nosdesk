<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import UserAutocomplete from "@/components/ticketComponents/UserSelection.vue";
import CustomDropdown from "@/components/ticketComponents/CustomDropdown.vue";
import { useDataStore } from '@/stores/dataStore';

const props = defineProps<{
  ticket: {
    id: number;
    title: string;
    status: string;
    priority: string;
    created?: string;
    modified?: string;
    assignee?: string;
    requester?: string;
  };
  createdDate: string;
  modifiedDate: string;
  selectedStatus: string;
  selectedPriority: string;
  statusOptions: { value: string; label: string }[];
  priorityOptions: { value: string; label: string }[];
}>();

const emit = defineEmits<{
  (e: "update:selectedStatus", value: string): void;
  (e: "update:selectedPriority", value: string): void;
  (e: "update:requester", value: string): void;
  (e: "update:assignee", value: string): void;
}>();

// Set up reactive state for requester and assignee
const selectedRequester = ref(props.ticket.requester || "");
const selectedAssignee = ref(props.ticket.assignee || "");

// Watch for changes in the ticket props to update the local state
watch(() => props.ticket.requester, (newRequester) => {
  if (newRequester !== undefined) {
    selectedRequester.value = newRequester;
  }
});

watch(() => props.ticket.assignee, (newAssignee) => {
  if (newAssignee !== undefined) {
    selectedAssignee.value = newAssignee;
  }
});

// Watch for changes in the selected requester and emit the update
watch(selectedRequester, (newRequester) => {
  if (newRequester !== props.ticket.requester) {
    console.log(`TicketDetails: Emitting update:requester with value: ${newRequester}`);
    emit("update:requester", newRequester);
  }
});

// Watch for changes in the selected assignee and emit the update
watch(selectedAssignee, (newAssignee) => {
  if (newAssignee !== props.ticket.assignee) {
    console.log(`TicketDetails: Emitting update:assignee with value: ${newAssignee}`);
    emit("update:assignee", newAssignee);
  }
});

// Component mounted
onMounted(() => {
  // Component initialization if needed
});
</script>

<template>
  <div class="w-full">
    <div class="bg-slate-800 rounded-2xl p-6">
      <h2 class="text-lg font-medium text-white mb-6">Ticket Details</h2>
      
      <div class="flex flex-col gap-6">
        <!-- Assignment Section -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- Requester -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Requester</h3>
            <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
              <UserAutocomplete
                v-model="selectedRequester"
                placeholder="Search or select Requester..."
                type="requester"
                class="w-full"
              />
            </div>
          </div>

          <!-- Assignee -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Assignee</h3>
            <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
              <UserAutocomplete
                v-model="selectedAssignee"
                placeholder="Search or select Assignee..."
                type="assignee"
                class="w-full"
              />
            </div>
          </div>
        </div>

        <!-- Status and Priority Section -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- Status -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Status</h3>
            <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
              <CustomDropdown
                :value="selectedStatus"
                :options="statusOptions"
                type="status"
                @update:value="emit('update:selectedStatus', $event)"
                class="w-full"
              />
            </div>
          </div>

          <!-- Priority -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Priority</h3>
            <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
              <CustomDropdown
                :value="selectedPriority"
                :options="priorityOptions"
                type="priority"
                @update:value="emit('update:selectedPriority', $event)"
                class="w-full"
              />
            </div>
          </div>
        </div>

        <!-- Timestamps Section -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- Created Date -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Created</h3>
            <div class="bg-slate-700/50 rounded-lg p-3 border border-slate-600/30">
              <span class="text-white text-sm">{{ createdDate }}</span>
            </div>
          </div>

          <!-- Modified Date -->
          <div class="flex flex-col gap-2">
            <h3 class="text-sm font-medium text-slate-300 uppercase tracking-wide">Last Modified</h3>
            <div class="bg-slate-700/50 rounded-lg p-3 border border-slate-600/30">
              <span class="text-white text-sm">{{ modifiedDate }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>