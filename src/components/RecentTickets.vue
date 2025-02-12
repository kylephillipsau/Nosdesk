# src/components/RecentTickets.vue
<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useRecentTicketsStore } from '@/stores/recentTickets'

const recentTicketsStore = useRecentTicketsStore()

const getStatusColor = (status: string) => {
  const colors = {
    'open': 'bg-yellow-500',
    'in-progress': 'bg-blue-500',
    'closed': 'bg-green-500'
  }
  return colors[status] || 'bg-gray-500'
}
</script>

<template>
  <div class="mt-8">
    <h3 class="px-4 text-sm font-medium text-gray-400 uppercase mb-2">Recent Tickets</h3>
    <div class="flex flex-col space-y-1">
      <RouterLink 
        v-for="ticket in recentTicketsStore.recentTickets"
        :key="ticket.id"
        :to="{
          path: `/tickets/${ticket.id}`,
          query: { fromRecent: 'true' }
        }"
        class="group px-4 py-2 rounded-lg transition-colors duration-200 hover:bg-slate-700 relative cursor-pointer"
      >
        <!-- Remove button -->
        <button
          @click.prevent="recentTicketsStore.removeRecentTicket(ticket.id)"
          class="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 rounded-full flex items-center justify-center text-gray-400 hover:text-white hover:bg-slate-600 opacity-0 group-hover:opacity-100 transition-all z-10"
        >
          ×
        </button>
        
        <div class="flex items-center justify-between pr-6">
          <div class="flex-1 min-w-0">
            <p class="text-sm text-white truncate">{{ ticket.title }}</p>
            <div class="flex items-center gap-2">
              <span 
                :class="[getStatusColor(ticket.status), 'px-1.5 py-0.5 rounded text-xs text-white']"
              >
                {{ ticket.status }}
              </span>
              <span class="text-xs text-gray-400">#{{ ticket.id }}</span>
            </div>
          </div>
        </div>
      </RouterLink>

      <!-- Empty state -->
      <div 
        v-if="recentTicketsStore.recentTickets.length === 0" 
        class="px-4 py-2 text-sm text-gray-400"
      >
        No recent tickets
      </div>
    </div>
  </div>
</template>