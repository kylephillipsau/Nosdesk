<!-- KanbanBoard.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

interface KanbanTicket {
  id: number;
  title: string;
  assignee: string;
  priority: 'low' | 'medium' | 'high';
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

// Initialize columns with some mock data
const columns = ref<KanbanColumn[]>([
  {
    id: 'todo',
    title: 'To Do',
    tickets: [
      { id: 1, title: 'Design user interface', assignee: 'Alice Smith', priority: 'high' },
      { id: 2, title: 'Set up CI/CD pipeline', assignee: 'Bob Johnson', priority: 'medium' }
    ]
  },
  {
    id: 'in-progress',
    title: 'In Progress',
    tickets: [
      { id: 3, title: 'Implement authentication', assignee: 'Charlie Brown', priority: 'high' }
    ]
  },
  {
    id: 'review',
    title: 'Review',
    tickets: [
      { id: 4, title: 'Code review: API endpoints', assignee: 'Diana Ross', priority: 'medium' }
    ]
  },
  {
    id: 'done',
    title: 'Done',
    tickets: [
      { id: 5, title: 'Project setup', assignee: 'Ethan Hunt', priority: 'low' }
    ]
  }
])

const draggingTicket = ref<{ columnId: string; ticketId: number } | null>(null)

const startDrag = (columnId: string, ticket: KanbanTicket) => {
  draggingTicket.value = { columnId, ticketId: ticket.id }
}

const onDrop = (targetColumnId: string) => {
  if (!draggingTicket.value) return

  const sourceColumn = columns.value.find(col => col.id === draggingTicket.value?.columnId)
  const targetColumn = columns.value.find(col => col.id === targetColumnId)
  
  if (!sourceColumn || !targetColumn) return

  const ticketIndex = sourceColumn.tickets.findIndex(t => t.id === draggingTicket.value?.ticketId)
  if (ticketIndex === -1) return

  const [ticket] = sourceColumn.tickets.splice(ticketIndex, 1)
  targetColumn.tickets.push(ticket)

  // TODO: Update ticket status in backend
  console.log(`Moved ticket ${ticket.id} from ${sourceColumn.id} to ${targetColumn.id}`)
  
  draggingTicket.value = null
}

const allowDrop = (e: DragEvent) => {
  e.preventDefault()
}

const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

const createTicket = (columnId: string) => {
  // TODO: Implement ticket creation
  console.log('Creating new ticket in column:', columnId)
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
  <div class="h-full flex flex-col">
    <div class="flex-1 overflow-x-auto overflow-y-auto">
      <div class="flex gap-4 p-4 min-w-max h-full">
        <div
          v-for="column in columns"
          :key="column.id"
          class="w-80 flex flex-col bg-slate-800 rounded-lg"
        >
          <!-- Column Header -->
          <div class="p-3 border-b border-slate-700 sticky top-0 bg-slate-800 z-10">
            <div class="flex items-center justify-between">
              <h3 class="font-medium text-slate-200">{{ column.title }}</h3>
              <span class="text-sm text-slate-400">{{ column.tickets.length }}</span>
            </div>
          </div>

          <!-- Column Content -->
          <div
            class="flex-1 p-2 overflow-y-auto"
            style="max-height: calc(100vh - 13rem);"
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
                  <div class="flex items-center justify-between">
                    <span class="text-xs text-slate-400">{{ ticket.assignee }}</span>
                    <span :class="[getPriorityColor(ticket.priority), 'text-xs px-2 py-0.5 rounded']">
                      {{ ticket.priority }}
                    </span>
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
  </div>
</template>

<style scoped>
/* Hide scrollbar for Chrome, Safari and Opera */
.overflow-y-auto::-webkit-scrollbar {
  display: none;
}

/* Hide scrollbar for IE, Edge and Firefox */
.overflow-y-auto {
  -ms-overflow-style: none;  /* IE and Edge */
  scrollbar-width: none;  /* Firefox */
}
</style> 