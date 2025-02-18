<!-- components/TicketDetails.vue -->
<script setup lang="ts">
import UserAvatar from "@/components/UserAvatar.vue";
import CustomDropdown from '@/components/CustomDropdown.vue'

const props = defineProps<{
  ticket: {
    id: number;
    title: string;
    status: "open" | "in-progress" | "closed";
    priority: "low" | "medium" | "high";
    created: string;
    assignee: string;
    requester: string;
  }
  formattedDate: string
  selectedStatus: "open" | "in-progress" | "closed"
  selectedPriority: "low" | "medium" | "high"
  statusOptions: { value: string; label: string }[]
  priorityOptions: { value: string; label: string }[]
}>()

const emit = defineEmits<{
  (e: 'update:selectedStatus', value: "open" | "in-progress" | "closed"): void
  (e: 'update:selectedPriority', value: "low" | "medium" | "high"): void
}>()
</script>

<template>
  <div class="w-full">
    <div class="flex flex-col bg-gray-800 rounded-2xl p-6 gap-4 shadow-lg">
      <h2 class="text-lg font-medium text-gray-100">Details</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Requester -->
        <div class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-gray-400">Requester</dt>
          <dd class="text-gray-200">
            <UserAvatar :name="ticket.requester" />
          </dd>
        </div>
        <!-- Assignee -->
        <div class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-gray-400">Assignee</dt>
          <dd class="text-gray-200">
            <UserAvatar :name="ticket.assignee" />
          </dd>
        </div>

        <!-- Created Date -->
        <div class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-gray-400">Created</dt>
          <dd class="text-gray-200">{{ formattedDate }}</dd>
        </div>

        <!-- Status -->
        <div class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-gray-400">Status</dt>
          <dd class="text-gray-200">
            <CustomDropdown :value="selectedStatus" :options="statusOptions" type="status"
              @update:value="emit('update:selectedStatus', $event as 'open' | 'in-progress' | 'closed')" />
          </dd>
        </div>

        <!-- Priority -->
        <div class="flex flex-col gap-1 bg-gray-700 p-3 rounded-xl shadow-inner">
          <dt class="text-sm text-gray-400">Priority</dt>
          <dd class="text-gray-200">
            <CustomDropdown :value="selectedPriority" :options="priorityOptions" type="priority"
              @update:value="emit('update:selectedPriority', $event as 'low' | 'medium' | 'high')" />
          </dd>
        </div>
      </div>
    </div>
  </div>
</template>