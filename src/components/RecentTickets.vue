// components/RecentTickets.vue
<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useRecentTicketsStore } from '@/stores/recentTickets'
import StatusBadge from '@/components/StatusBadge.vue'

const recentTicketsStore = useRecentTicketsStore()
</script>

<template>
  <div class="h-full flex flex-col">
    <h3 class="px-4 text-sm font-medium text-gray-400 uppercase mb-2 flex-shrink-0">Recent Tickets</h3>

    <div class="flex-1 min-h-0 overflow-y-auto px-2">
      <div class="space-y-1">
        <RouterLink 
          v-for="ticket in recentTicketsStore.recentTickets" 
          :key="ticket.id" 
          :to="{
            path: `/tickets/${ticket.id}`,
            query: { fromRecent: 'true' }
          }"
          class="group px-2 py-1.5 rounded-lg transition-colors duration-200 hover:bg-slate-700 relative cursor-pointer flex items-center justify-between"
        >
          <!-- Left side content -->
          <div class="flex items-center gap-2 flex-1 min-w-0">
            <StatusBadge 
              type="status" 
              :value="ticket.status"
              custom-classes="w-2.5 h-2.5 rounded-full"
            />
            <span 
              class="text-sm text-white truncate relative"
              :title="ticket.title"
            >
              {{ ticket.title }}
              <!-- Tooltip for full title -->
              <span 
                class="absolute invisible group-hover:visible opacity-0 group-hover:opacity-100 
                       transition-all duration-100 bg-black text-white text-xs px-2 py-1 rounded
                       -top-8 left-0 whitespace-nowrap z-10 shadow-lg"
              >
                {{ ticket.title }}
              </span>
            </span>
            <span class="text-xs text-gray-400">#{{ ticket.id }}</span>
          </div>

          <!-- Remove button -->
          <button 
            @click.prevent="recentTicketsStore.removeRecentTicket(ticket.id)"
            class="w-5 h-5 rounded-full flex items-center justify-center text-gray-400 hover:text-white hover:bg-slate-600 opacity-0 group-hover:opacity-100 transition-all shrink-0 ml-2"
          >
            ×
          </button>
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
  </div>
</template>