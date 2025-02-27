// views/Dashboard.vue
<script setup lang="ts">
import { ref } from 'vue'
import TicketHeatmap from '@/components/TicketHeatmap.vue'

// Get current time for greeting
const getGreeting = () => {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 18) return 'Good afternoon'
  return 'Good evening'
}

const greeting = ref(getGreeting())
const username = ref('Kyle') // This would come from your auth system

// Mock ticket stats - replace with actual data
const ticketStats = ref({
  total: 12,
  open: 5,
  inProgress: 4,
  closed: 3
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Content -->
    <div class="flex flex-col gap-4 p-6">
      <!-- Greeting Card -->
      <div class="mb-6">
        <h2 class="text-3xl font-medium text-white">
          {{ greeting }}, {{ username }}!
        </h2>
        <p class="text-gray-400 mt-2">
          Welcome to your nosDesk dashboard
        </p>
      </div>

      <!-- Stats Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Total Tickets -->
        <div class="bg-slate-800 rounded-lg p-6">
          <h3 class="text-gray-400 text-sm font-medium">Total Tickets</h3>
          <p class="text-2xl font-semibold text-white mt-2">{{ ticketStats.total }}</p>
        </div>

        <!-- Open Tickets -->
        <div class="bg-slate-800 rounded-lg p-6">
          <h3 class="text-gray-400 text-sm font-medium">Open Tickets</h3>
          <p class="text-2xl font-semibold text-green-500 mt-2">{{ ticketStats.open }}</p>
        </div>

        <!-- In Progress -->
        <div class="bg-slate-800 rounded-lg p-6">
          <h3 class="text-gray-400 text-sm font-medium">In Progress</h3>
          <p class="text-2xl font-semibold text-blue-500 mt-2">{{ ticketStats.inProgress }}</p>
        </div>

        <!-- Closed -->
        <div class="bg-slate-800 rounded-lg p-6">
          <h3 class="text-gray-400 text-sm font-medium">Closed</h3>
          <p class="text-2xl font-semibold text-gray-400 mt-2">{{ ticketStats.closed }}</p>
        </div>
      </div>
      <TicketHeatmap />

    </div>

  </div>
</template>