<!-- KanbanBoard.vue -->
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { projectService } from '@/services/projectService'
import ticketService from '@/services/ticketService'
import { STATUS_OPTIONS, type TicketStatus } from '@/constants/ticketOptions'
import UserAvatar from '@/components/UserAvatar.vue'
import AddTicketToProjectModal from './AddTicketToProjectModal.vue'

interface KanbanTicket {
  id: number;
  title: string;
  assignee: string;
  priority: 'low' | 'medium' | 'high';
  status: string;
}

interface KanbanColumn {
  id: string;
  title: string;
  tickets: KanbanTicket[];
}

const props = defineProps<{
  projectId: number;
}>()

const router = useRouter()
const isLoading = ref(true)
const error = ref<string | null>(null)

// Initialize columns with empty arrays
const columns = ref<KanbanColumn[]>([
  {
    id: 'open',
    title: 'Open',
    tickets: []
  },
  {
    id: 'in-progress',
    title: 'In Progress',
    tickets: []
  },
  {
    id: 'closed',
    title: 'Closed',
    tickets: []
  }
])

const draggingTicket = ref<{ columnId: string; ticketId: number } | null>(null)
const showAddTicketModal = ref(false)
const currentColumnId = ref<string | null>(null)
const projectTicketIds = ref<number[]>([])

// Fetch project tickets
const fetchProjectTickets = async () => {
  if (!props.projectId) return
  
  try {
    isLoading.value = true
    error.value = null
    
    // Fetch tickets for the project
    const tickets = await projectService.getProjectTickets(props.projectId)
    
    // Store all ticket IDs for the project to avoid duplicates
    projectTicketIds.value = tickets.map(ticket => ticket.id)
    
    // Reset all columns
    columns.value.forEach(column => {
      column.tickets = []
    })
    
    // Distribute tickets to appropriate columns based on status
    tickets.forEach(ticket => {
      const kanbanTicket: KanbanTicket = {
        id: ticket.id,
        title: ticket.title,
        assignee: ticket.assignee || 'Unassigned',
        priority: ticket.priority as 'low' | 'medium' | 'high',
        status: ticket.status
      }
      
      // Map ticket status to column
      let columnId: string
      switch (ticket.status) {
        case 'in-progress':
          columnId = 'in-progress'
          break
        case 'closed':
          columnId = 'closed'
          break
        case 'open':
        default:
          columnId = 'open'
          break
      }
      
      // Find the column and add the ticket
      const column = columns.value.find(col => col.id === columnId)
      if (column) {
        column.tickets.push(kanbanTicket)
      }
    })
  } catch (err) {
    console.error('Failed to fetch project tickets:', err)
    error.value = 'Failed to load tickets. Please try again later.'
  } finally {
    isLoading.value = false
  }
}

// Watch for changes to projectId
watch(() => props.projectId, (newProjectId) => {
  if (newProjectId) {
    fetchProjectTickets()
  }
}, { immediate: true })

// Fetch tickets on component mount
onMounted(() => {
  if (props.projectId) {
    fetchProjectTickets()
  }
})

const startDrag = (columnId: string, ticket: KanbanTicket) => {
  draggingTicket.value = { columnId, ticketId: ticket.id }
}

const onDrop = async (targetColumnId: string) => {
  if (!draggingTicket.value) return

  const sourceColumn = columns.value.find(col => col.id === draggingTicket.value?.columnId)
  const targetColumn = columns.value.find(col => col.id === targetColumnId)
  
  if (!sourceColumn || !targetColumn) return

  const ticketIndex = sourceColumn.tickets.findIndex(t => t.id === draggingTicket.value?.ticketId)
  if (ticketIndex === -1) return

  const [ticket] = sourceColumn.tickets.splice(ticketIndex, 1)
  targetColumn.tickets.push(ticket)

  // Map column ID to ticket status
  let newStatus: TicketStatus
  switch (targetColumnId) {
    case 'in-progress':
      newStatus = 'in-progress'
      break
    case 'closed':
      newStatus = 'closed'
      break
    case 'open':
    default:
      newStatus = 'open'
      break
  }
  
  // Update ticket status in backend
  try {
    await ticketService.updateTicket(ticket.id, { 
      status: newStatus,
      modified: new Date().toISOString()
    })
    console.log(`Updated ticket ${ticket.id} status to ${newStatus}`)
  } catch (err) {
    console.error(`Failed to update ticket status:`, err)
    // Revert the UI change if the API call fails
    // This would require refetching the tickets
    await fetchProjectTickets()
  }
  
  draggingTicket.value = null
}

const allowDrop = (e: DragEvent) => {
  e.preventDefault()
}

const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

const createTicket = async (columnId: string) => {
  currentColumnId.value = columnId
  showAddTicketModal.value = true
}

const handleAddTicket = (ticketId: number) => {
  console.log(`Ticket ${ticketId} added to project ${props.projectId}`)
  fetchProjectTickets()
}

const getPriorityColor = (priority: string) => {
  switch (priority) {
    case 'high':
      return 'bg-red-500/20 text-red-400'
    case 'medium':
      return 'bg-yellow-500/20 text-yellow-400'
    case 'low':
      return 'bg-green-500/20 text-green-400'
    default:
      return 'bg-slate-500/20 text-slate-400'
  }
}
</script>

<template>
  <div class="h-full flex flex-col relative">
    <!-- Error message -->
    <div v-if="error" class="bg-red-500/20 border border-red-500/50 text-red-200 px-4 py-3 rounded-lg mb-4">
      {{ error }}
    </div>
    
    <!-- Loading state -->
    <div v-if="isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
    </div>
    
    <div v-else class="flex-1 overflow-x-auto">
      <div class="flex gap-4 p-4 min-w-max h-full">
        <div
          v-for="column in columns"
          :key="column.id"
          class="w-80 flex flex-col bg-slate-800 rounded-lg h-full"
        >
          <!-- Column Header -->
          <div class="p-3 border-b border-slate-700 sticky top-0 bg-slate-800" style="z-index: 1;">
            <div class="flex items-center justify-between">
              <h3 class="font-medium text-slate-200">{{ column.title }}</h3>
              <span class="text-sm text-slate-400">{{ column.tickets.length }}</span>
            </div>
          </div>

          <!-- Column Content -->
          <div
            class="flex-1 p-2 overflow-y-auto hide-vertical-scrollbar gap-2"
            @dragover="allowDrop"
            @drop="onDrop(column.id)"
          >
            <!-- Tickets -->
            <div class="flex flex-col gap-3">
              <div
                v-for="ticket in column.tickets"
                :key="ticket.id"
                class="bg-slate-700 p-3 rounded shadow-sm cursor-move hover:bg-slate-600 transition-colors"
                draggable="true"
                @dragstart="startDrag(column.id, ticket)"
                @click="openTicket(ticket.id)"
              >
                <div class="flex flex-col gap-2">
                  <h4 class="text-sm font-medium text-slate-200">{{ ticket.title }}</h4>
                  <div class="flex flex-col gap-2">
                    <div class="flex items-center justify-between">
                      <UserAvatar 
                        v-if="ticket.assignee && ticket.assignee !== 'Unassigned'" 
                        :name="ticket.assignee" 
                        size="xs" 
                        :showName="true"
                        :clickable="false"
                      />
                      <span v-else class="text-xs text-slate-400">Unassigned</span>
                      <span :class="[getPriorityColor(ticket.priority), 'text-xs px-2 py-0.5 rounded']">
                        {{ ticket.priority }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Add Ticket Button -->
            <button
              @click="createTicket(column.id)"
              class="w-full mt-3 p-2 text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-600/50 rounded transition-colors"
            >
              + Add ticket
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Add Ticket Modal -->
    <AddTicketToProjectModal 
      :show="showAddTicketModal"
      :project-id="props.projectId"
      :existing-tickets="projectTicketIds"
      @close="showAddTicketModal = false"
      @add-ticket="handleAddTicket"
      @refresh="fetchProjectTickets"
    />
  </div>
</template>

<style scoped>
/* Only hide vertical scrollbars for ticket columns */
.hide-vertical-scrollbar::-webkit-scrollbar {
  display: none;
}

.hide-vertical-scrollbar {
  -ms-overflow-style: none;  /* IE and Edge */
  scrollbar-width: none;  /* Firefox */
}

/* Style horizontal scrollbar */
.overflow-x-auto::-webkit-scrollbar {
  height: 12px;
  display: block;
}

.overflow-x-auto::-webkit-scrollbar-track {
  background: rgb(30 41 59); /* slate-800 */
}

.overflow-x-auto::-webkit-scrollbar-thumb {
  background: rgb(51 65 85); /* slate-700 */
  border-radius: 4px;
}

.overflow-x-auto::-webkit-scrollbar-thumb:hover {
  background: rgb(71 85 105); /* slate-600 */
}

.overflow-x-auto {
  scrollbar-color: rgb(51 65 85) rgb(30 41 59); /* thumb and track colors */
}

/* Ensure kanban content stays below main header */
.relative {
  z-index: 0;
}
</style> 