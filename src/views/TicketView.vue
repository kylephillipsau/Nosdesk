<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useRecentTicketsStore } from '@/stores/recentTickets'
import UserAvatar from '@/components/UserAvatar.vue'

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  priority: 'low' | 'medium' | 'high';
  created: string;
  assignee: string;
}

const route = useRoute()
const router = useRouter()
const ticket = ref<Ticket | null>(null)
const recentTicketsStore = useRecentTicketsStore()

const fetchTicket = async (ticketId: string | string[]) => {
  const id = Number(ticketId)
  const ticketData = (await import('@/assets/tickets.json')).default
  const foundTicket = ticketData.tickets.find(t => t.id === id)
  
  if (!foundTicket) {
    router.push('/404')
    return
  }
  
  ticket.value = foundTicket

  // Check if we navigated from the recent tickets list
  const fromRecent = route.query.fromRecent === 'true'
  recentTicketsStore.addRecentTicket({
    id: foundTicket.id,
    title: foundTicket.title,
    status: foundTicket.status
  }, fromRecent)
}

// Watch for route changes
watch(
  () => route.params.id,
  (newId) => {
    if (newId) {
      fetchTicket(newId)
    }
  },
  { immediate: true } // This will run on component mount too
)

const getStatusColor = (status: string) => {
  const colors = {
    'open': 'bg-yellow-500',
    'in-progress': 'bg-blue-500',
    'closed': 'bg-green-500'
  }
  return colors[status] || 'bg-gray-500'
}

const getPriorityColor = (priority: string) => {
  const colors = {
    'low': 'text-green-400',
    'medium': 'text-yellow-400',
    'high': 'text-red-400'
  }
  return colors[priority] || 'text-gray-400'
}
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="p-6 max-w-4xl mx-auto">
      <!-- Back button -->
      <button 
        @click="router.back()"
        class="mb-6 flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
        <span>←</span> Back to tickets
      </button>

      <!-- Ticket header -->
      <div class="flex items-start justify-between mb-8">
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h1 class="text-2xl font-semibold text-white">{{ ticket.title }}</h1>
            <span class="text-gray-400">#{{ ticket.id }}</span>
          </div>
          <div class="flex items-center gap-3">
            <span 
              :class="[getStatusColor(ticket.status), 'px-3 py-1 rounded-full text-sm']">
              {{ ticket.status }}
            </span>
            <span 
              :class="[getPriorityColor(ticket.priority), 'font-medium']">
              {{ ticket.priority }} priority
            </span>
          </div>
        </div>
        
        <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors text-white">
          Edit Ticket
        </button>
      </div>

      <!-- Ticket details -->
      <div class="grid grid-cols-2 gap-6">
        <div class="bg-gray-800/50 rounded-lg p-6">
          <h2 class="text-lg font-medium text-white mb-4">Details</h2>
          <dl class="grid grid-cols-[120px,1fr] gap-y-4 text-sm">
            <dt class="text-gray-400">Assignee</dt>
            <dd class="text-white"><UserAvatar :name="ticket.assignee" />
            </dd>
            
            <dt class="text-gray-400">Created</dt>
            <dd class="text-white">{{ ticket.created }}</dd>
            
            <dt class="text-gray-400">Status</dt>
            <dd class="text-white capitalize">{{ ticket.status }}</dd>
            
            <dt class="text-gray-400">Priority</dt>
            <dd class="text-white capitalize">{{ ticket.priority }}</dd>
          </dl>
        </div>
      </div>
    </div>

    <!-- Loading state -->
    <div v-else class="p-6 text-center text-gray-400">
      Loading ticket...
    </div>
  </div>
</template>