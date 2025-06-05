// components/RecentTickets.vue
<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useRecentTicketsStore } from '@/stores/recentTickets'
import StatusBadge from '@/components/StatusBadge.vue'
import QuickTooltip from '@/components/QuickTooltip.vue'
import { ref } from 'vue'

const recentTicketsStore = useRecentTicketsStore()
const draggedTicketId = ref<number | null>(null)

const formatDate = (dateString: string | undefined) => {
  if (!dateString) return ''
  const date = new Date(dateString)
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

const handleDragStart = (ticketId: number, dragEvent: DragEvent) => {
  draggedTicketId.value = ticketId
  if (dragEvent.dataTransfer) {
    dragEvent.dataTransfer.effectAllowed = 'move'
  }
}

const handleDragEnd = () => {
  draggedTicketId.value = null
}

const handleDragOver = (ticketId: number, dragEvent: DragEvent) => {
  dragEvent.preventDefault()
  if (draggedTicketId.value === ticketId) return
  
  const tickets = [...recentTicketsStore.recentTickets]
  const draggedIndex = tickets.findIndex(ticket => ticket.id === draggedTicketId.value)
  const targetIndex = tickets.findIndex(ticket => ticket.id === ticketId)
  
  if (draggedIndex !== -1 && targetIndex !== -1) {
    const [draggedTicket] = tickets.splice(draggedIndex, 1)
    tickets.splice(targetIndex, 0, draggedTicket)
    recentTicketsStore.$patch({ recentTickets: tickets })
  }
}

const handleDrop = () => {
  handleDragEnd()
}
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex-1 min-h-0 overflow-y-auto">
      <div class="flex flex-col gap-0.5 pt-1 pb-1 px-1">
        <!-- Wrap each ticket item with QuickTooltip -->
        <div v-for="ticket in recentTicketsStore.recentTickets" :key="ticket.id" class="block">
          <QuickTooltip 
            :text="ticket.title" 
            :details="{
              title: ticket.title,
              status: ticket.status,
              requester: ticket.requester,
              assignee: ticket.assignee,
              created: formatDate(ticket.created)
            }"
            :disabled="draggedTicketId !== null"
            :fullWidth="true"
          >
            <RouterLink 
              :to="{
                path: `/tickets/${ticket.id}`,
                query: { fromRecent: 'true' }
              }"
              draggable="true"
              @dragstart="(dragEvent: DragEvent) => handleDragStart(ticket.id, dragEvent)"
              @dragend="handleDragEnd"
              @dragover="(dragEvent: DragEvent) => handleDragOver(ticket.id, dragEvent)"
              @drop="handleDrop"
              class="group block px-2 py-1 rounded-md hover:bg-slate-700/70 relative cursor-move transition-all duration-200 border border-transparent"
              :class="{
                'opacity-60': draggedTicketId === ticket.id,
                'shadow-sm border-blue-500/20 bg-blue-900/10 hover:bg-blue-900/20': ticket.isDraft,
                'hover:shadow-sm': !ticket.isDraft
              }"
            >
              <!-- Compact layout with status badge, ID, and title in a single row -->
              <div class="flex items-center w-full pr-6 gap-1">
                <!-- Status Badge -->
                <StatusBadge 
                  type="status" 
                  :value="ticket.status"
                  custom-classes="w-1.5 h-1.5 rounded-full flex-shrink-0 mr-1.5"
                />
                
                <!-- Ticket ID -->
                <span class="text-xs font-mono text-slate-400 whitespace-nowrap mr-2">{{ ticket.id }}</span>
                
                <!-- Ticket Title -->
                <span class="text-xs text-white truncate block min-w-0 flex-1">
                  {{ ticket.title }}
                  <span v-if="ticket.isDraft" class="ml-1 text-xs text-blue-400 font-medium">(Draft)</span>
                </span>
              </div>
              
              <!-- Action Controls Container - Right Side -->
              <div class="absolute right-1 inset-y-0 flex items-center">
                <!-- Remove button - Only show on hover -->
                <button 
                  @click.prevent="recentTicketsStore.removeRecentTicket(ticket.id)"
                  class="w-3.5 h-3.5 rounded-full flex items-center justify-center text-slate-500 hover:text-white hover:bg-slate-600 opacity-0 group-hover:opacity-100 transition-all flex-shrink-0"
                  title="Remove from recent tickets"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-2.5 w-2.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </RouterLink>
          </QuickTooltip>
        </div>

        <!-- Empty state - More compact -->
        <div 
          v-if="recentTicketsStore.recentTickets.length === 0" 
          class="px-2 py-1.5 text-xs text-slate-400 bg-slate-800/40 text-center"
        >
          No recent tickets
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transform {
  will-change: transform;
}

/* Add subtle grip pattern to draggable items */
[draggable="true"] {
  background-image: linear-gradient(
    to right,
    transparent,
    transparent
  );
  background-position: right center;
  background-repeat: no-repeat;
  background-size: 8px 100%;
}

[draggable="true"]:hover {
  background-image: linear-gradient(
    to right,
    transparent,
    transparent
  );
}

[draggable="true"]:active {
  cursor: grabbing;
}
</style>