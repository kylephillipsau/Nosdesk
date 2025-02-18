<script setup lang="ts">
import { ref, computed } from 'vue';
import UserAvatar from "@/components/UserAvatar.vue";
import CustomDropdown from '@/components/CustomDropdown.vue';
import UserSelectionDialog from '@/components/UserSelectionDialog.vue';
import users from '@/assets/users.json'; // Assuming this is your correct path

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
  // Removing userOptions since we're using full user data now
}>()

const emit = defineEmits<{
  (e: 'update:selectedStatus', value: "open" | "in-progress" | "closed"): void
  (e: 'update:selectedPriority', value: "low" | "medium" | "high"): void
  (e: 'update:requester', value: string): void // New emit for updating requester
  (e: 'update:assignee', value: string): void  // New emit for updating assignee
}>()

// State to control dialog visibility
const isUserDialogOpen = ref(false);
const userType = ref<'requester' | 'assignee'>('requester'); // or 'assignee'

const openUserDialog = (type: 'requester' | 'assignee') => {
  userType.value = type;
  isUserDialogOpen.value = true;
};

const handleUserSelection = (userId: string) => {
  if (userType.value === 'requester') {
    emit('update:requester', userId);
  } else {
    emit('update:assignee', userId);
  }
  isUserDialogOpen.value = false; // Close the dialog after selection
};
</script>

<template>
  <div class="w-full">
    <div class="flex flex-col bg-slate-800 rounded-2xl p-6 gap-4 shadow-lg">
      <h2 class="text-lg font-medium text-slate-100">Details</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Requester -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Requester</dt>
          <dd class="text-slate-200">
            <div @click="openUserDialog('requester')" class="cursor-pointer">
              <UserAvatar :name="ticket.requester" />
            </div>
          </dd>
        </div>
        <!-- Assignee -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Assignee</dt>
          <dd class="text-slate-200">
            <div @click="openUserDialog('assignee')" class="cursor-pointer">
              <UserAvatar :name="ticket.assignee" />
            </div>
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
            <CustomDropdown :value="selectedStatus" :options="statusOptions" type="status"
              @update:value="emit('update:selectedStatus', $event as 'open' | 'in-progress' | 'closed')" />
          </dd>
        </div>

        <!-- Priority -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-slate-400">Priority</dt>
          <dd class="text-slate-200">
            <CustomDropdown :value="selectedPriority" :options="priorityOptions" type="priority"
              @update:value="emit('update:selectedPriority', $event as 'low' | 'medium' | 'high')" />
          </dd>
        </div>
      </div>
    </div>

    <!-- User Selection Dialog -->
    <UserSelectionDialog 
      :isOpen="isUserDialogOpen" 
      :users="usersFromJson" 
      @update:isOpen="isUserDialogOpen = $event"
      @selectUser="handleUserSelection"
    />
  </div>
</template>