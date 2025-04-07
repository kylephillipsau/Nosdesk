// views/Dashboard.vue
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import TicketHeatmap from '@/components/TicketHeatmap.vue'
import UserAssignedTickets from '@/components/UserAssignedTickets.vue'
import { getTickets } from '@/services/ticketService'
import type { Ticket } from '@/services/ticketService'
import { useAuthStore } from '@/stores/auth'

// Initialize auth store
const authStore = useAuthStore()

// Get current time for greeting
const getGreeting = () => {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 18) return 'Good afternoon'
  return 'Good evening'
}

const greeting = ref(getGreeting())

// Get username from auth store
const username = computed(() => {
  if (!authStore.user?.name) return 'Guest';
  
  // Split the full name and take the first part as the first name
  const firstName = authStore.user.name.split(' ')[0];
  return firstName;
})

// Initialize ticket stats with zeros
const ticketStats = ref({
  total: 0,
  open: 0,
  inProgress: 0,
  closed: 0
})

// Fetch tickets and update stats
const fetchTicketStats = async () => {
  try {
    const tickets = await getTickets()
    
    // Calculate stats
    ticketStats.value = {
      total: tickets.length,
      open: tickets.filter(ticket => ticket.status === 'open').length,
      inProgress: tickets.filter(ticket => ticket.status === 'in-progress').length,
      closed: tickets.filter(ticket => ticket.status === 'closed').length
    }
  } catch (error) {
    console.error('Error fetching ticket stats:', error)
  }
}

// Fetch data when component mounts
onMounted(() => {
  fetchTicketStats()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Content -->
    <div class="flex flex-col gap-6 p-6">
      <!-- Greeting Card -->
      <div class="mb-2">
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

      <!-- Two column layout for assigned tickets and heatmap -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Assigned Tickets -->
        <div class="lg:col-span-2">
          <UserAssignedTickets :limit="5" />
        </div>

        <!-- Ticket Activity -->
        <div>
          <div class="bg-slate-800 rounded-lg p-6">
            <h3 class="text-lg font-medium text-white mb-4">Ticket Activity</h3>
            <TicketHeatmap ticketStatus="closed" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>