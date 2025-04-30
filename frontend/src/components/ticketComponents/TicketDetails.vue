<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import UserAutocomplete from "@/components/ticketComponents/UserSelection.vue";
import CustomDropdown from "@/components/ticketComponents/CustomDropdown.vue";

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
  users?: { id: string; name: string; email: string }[]; // Make users a prop instead of importing from JSON
}>();

const emit = defineEmits<{
  (e: "update:selectedStatus", value: string): void;
  (e: "update:selectedPriority", value: string): void;
  (e: "update:requester", value: string): void;
  (e: "update:assignee", value: string): void;
}>();

// Use the users prop with a fallback to an empty array if not provided
const usersFromProps = computed(() => props.users || []);

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
</script>

<template>
  <div class="w-full">
    <div class="flex flex-col bg-slate-800 rounded-xl pt-3 p-2 gap-2 shadow-lg">
      <h2 class="text-lg font-medium text-slate-100 px-2">Details</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <!-- Requester -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Requester</dt>
          <dd class="text-slate-200">
            <UserAutocomplete
              v-model="selectedRequester"
              :users="usersFromProps"
              placeholder="Search or select Requester..."
              type="requester"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Assignee -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Assignee</dt>
          <dd class="text-slate-200">
            <UserAutocomplete
              v-model="selectedAssignee"
              :users="usersFromProps"
              placeholder="Search or select Assignee..."
              type="assignee"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Created Date -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Created</dt>
          <dd class="text-slate-200 px-1">{{ createdDate }}</dd>
        </div>

        <!-- Date Modified -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Modified</dt>
          <dd class="text-slate-200 px-1">{{ modifiedDate }}</dd>
        </div>

        <!-- Status -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Status</dt>
          <dd class="text-slate-200">
            <CustomDropdown
              :value="selectedStatus"
              :options="statusOptions"
              type="status"
              @update:value="emit('update:selectedStatus', $event)"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Priority -->
        <div class="flex flex-col gap-1 bg-slate-700 p-2 rounded-md shadow-inner">
          <dt class="text-sm text-slate-400 px-1">Priority</dt>
          <dd class="text-slate-200">
            <CustomDropdown
              :value="selectedPriority"
              :options="priorityOptions"
              type="priority"
              @update:value="emit('update:selectedPriority', $event)"
              class="w-full"
            />
          </dd>
        </div>
      </div>
    </div>
  </div>
</template>