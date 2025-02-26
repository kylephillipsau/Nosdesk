<script setup lang="ts">
import { ref, computed } from 'vue';
import UserAutocomplete from "@/components/ticketComponents/UserSelection.vue";
import CustomDropdown from '@/components/ticketComponents/CustomDropdown.vue';
import users from '@/data/users.json'; // Assuming this is your correct path

const usersFromJson = computed(() => users.users);

const props = defineProps<{
  ticket: {
    id: number;
    title: string;
    status: "open" | "in-progress" | "closed";
    priority: "low" | "medium" | "high";
    created: string;
    modified: string;
    assignee: string;
    requester: string;
  }
  createdDate: string
  modifiedDate: string
  selectedStatus: "open" | "in-progress" | "closed"
  selectedPriority: "low" | "medium" | "high"
  statusOptions: { value: string; label: string }[]
  priorityOptions: { value: string; label: string }[]
}>()

const emit = defineEmits<{
  (e: 'update:selectedStatus', value: "open" | "in-progress" | "closed"): void
  (e: 'update:selectedPriority', value: "low" | "medium" | "high"): void
  (e: 'update:requester', value: string): void // Emit for updating requester
  (e: 'update:assignee', value: string): void  // Emit for updating assignee
}>()

// State for selected user IDs (bound via v-model in UserAutocomplete)
const selectedRequester = ref(props.ticket.requester);
const selectedAssignee = ref(props.ticket.assignee);
</script>

<template>
  <div class="w-full">
    <div class="flex flex-col bg-slate-800 rounded-2xl p-4 gap-2 shadow-lg">
      <h2 class="text-lg font-medium text-slate-100">Details</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Requester -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Requester</dt>
          <dd class="text-slate-200">
            <UserAutocomplete
              v-model="selectedRequester"
              :users="usersFromJson"
              placeholder="Search or select Requester..."
              type="requester"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Assignee -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Assignee</dt>
          <dd class="text-slate-200">
            <UserAutocomplete
              v-model="selectedAssignee"
              :users="usersFromJson"
              placeholder="Search or select Assignee..."
              type="assignee"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Created Date -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Created</dt>
          <dd class="text-slate-200">{{ createdDate }}</dd>
        </div>

        <!-- Date Modified -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Modified</dt>
          <dd class="text-slate-200">{{ modifiedDate }}</dd>
        </div>

        <!-- Status -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Status</dt>
          <dd class="text-slate-200">
            <CustomDropdown
              :value="selectedStatus"
              :options="statusOptions"
              type="status"
              @update:value="emit('update:selectedStatus', $event as 'open' | 'in-progress' | 'closed')"
              class="w-full"
            />
          </dd>
        </div>

        <!-- Priority -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Priority</dt>
          <dd class="text-slate-200">
            <CustomDropdown
              :value="selectedPriority"
              :options="priorityOptions"
              type="priority"
              @update:value="emit('update:selectedPriority', $event as 'low' | 'medium' | 'high')"
              class="w-full"
            />
          </dd>
        </div>
      </div>
    </div>
  </div>
</template>