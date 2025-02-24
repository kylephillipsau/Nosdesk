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
    <h3 class="px-4 text-sm font-medium text-gray-400 uppercase mb-2 flex-shrink-0">Recent Tickets</h3>

    <div class="flex-1 min-h-0 overflow-y-auto">
      <div class="space-y-1 pt-2 px-2">
        <RouterLink 
          v-for="ticket in recentTicketsStore.recentTickets" 
          :key="ticket.id" 
          :to="{
            path: `/tickets/${ticket.id}`,
            query: { fromRecent: 'true' }
          }"
          draggable="true"
          @dragstart="(dragEvent: DragEvent) => handleDragStart(ticket.id, dragEvent)"
          @dragend="handleDragEnd"
          @dragover="(dragEvent: DragEvent) => handleDragOver(ticket.id, dragEvent)"
          @drop="handleDrop"
          class="group block px-2 py-1.5 rounded-lg hover:bg-slate-700 relative cursor-move transition-all duration-200"
          :class="{
            'opacity-50': draggedTicketId === ticket.id
          }"
        >
          <div class="flex items-center gap-2 w-full">
            <StatusBadge 
              type="status" 
              :value="ticket.status"
              custom-classes="w-2.5 h-2.5 rounded-full flex-shrink-0"
            />
            
            <div class="flex items-center min-w-0 flex-1">
              <QuickTooltip 
                :text="ticket.title" 
                :details="{
                  title: ticket.title,
                  status: ticket.status,
                  requester: ticket.requester,
                  assignee: ticket.assignee,
                  created: formatDate(ticket.created)
                }"
                class="min-w-0 flex-1"
                :disabled="draggedTicketId !== null"
              >
                <span class="text-sm text-white truncate block">
                  {{ ticket.title }}
                </span>
              </QuickTooltip>

              <div class="flex items-center gap-2 flex-shrink-0 ml-2">
                <span class="text-xs text-gray-400 whitespace-nowrap">#{{ ticket.id }}</span>
                <button 
                  @click.prevent="recentTicketsStore.removeRecentTicket(ticket.id)"
                  class="w-5 h-5 rounded-full flex items-center justify-center text-gray-400 hover:text-white hover:bg-slate-600 opacity-0 group-hover:opacity-100 transition-all flex-shrink-0"
                >
                  ×
                </button>
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
  </div>
</template>

<style scoped>
.transform {
  will-change: transform;
}
</style>