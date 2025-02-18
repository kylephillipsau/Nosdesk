// views/ListView.vue
<script setup lang="ts">
import ticketData from '@/assets/tickets.json'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import UserAvatar from '@/components/UserAvatar.vue'
import StatusBadge from '@/components/StatusBadge.vue'
import PageHeader from '@/components/PageHeader.vue'

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  priority: 'low' | 'medium' | 'high';
  created: string;
  assignee: string;
}

const tickets = ticketData.tickets as Ticket[]
const selectedTickets = ref<number[]>([])

const toggleSelection = (event: Event, ticketId: number) => {
  event.stopPropagation()
  const index = selectedTickets.value.indexOf(ticketId)
  if (index === -1) {
    selectedTickets.value.push(ticketId)
  } else {
    selectedTickets.value.splice(index, 1)
  }
}

const toggleAllTickets = (event: Event) => {
  event.stopPropagation()
  const checkbox = event.target as HTMLInputElement
  if (checkbox.checked) {
    selectedTickets.value = tickets.map(ticket => ticket.id)
  } else {
    selectedTickets.value = []
  }
}

const router = useRouter()

const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

const createNewTicket = () => {
  // Implement your new ticket creation logic here
  // For example, navigate to a new ticket form:
  router.push('/tickets/new')
  // Or open a modal, etc.
}
</script>

<template>
  <div class="flex flex-col h-full">
    <PageHeader title="Tickets List">
      <template #actions>
        <button 
          class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors text-white"
          @click="createNewTicket"
        >
          New Ticket
        </button>
      </template>
    </PageHeader>

    <!-- Table Container with adjusted width handling -->
    <div class="flex-1 overflow-auto">
      <div class="min-w-full inline-block align-middle">
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-slate-800 text-sm text-gray-200">
            <thead class="bg-slate-700/50">
              <tr>
                <th scope="col" class="p-4 w-10">
                  <input type="checkbox"
                    class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                    :checked="selectedTickets.length === tickets.length" 
                    @change="toggleAllTickets">
                </th>
                <th scope="col" class="text-left p-4 font-medium w-20">ID</th>
                <th scope="col" class="text-left p-4 font-medium">Title</th>
                <th scope="col" class="text-left p-4 font-medium w-24">Status</th>
                <th scope="col" class="text-left p-4 font-medium w-24">Priority</th>
                <th scope="col" class="text-left p-4 font-medium w-32">Created</th>
                <th scope="col" class="text-left p-4 font-medium w-32">Assignee</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-800">
              <tr v-for="ticket in tickets" 
                  :key="ticket.id"
                  class="hover:bg-slate-800/50 transition-colors cursor-pointer"
                  @click="openTicket(ticket.id)">
                <td class="px-4 py-1">
                  <input type="checkbox"
                    class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                    :checked="selectedTickets.includes(ticket.id)" 
                    @change="(e) => toggleSelection(e, ticket.id)">
                </td>
                <td class="px-4 py-1 whitespace-nowrap">#{{ ticket.id }}</td>
                <td class="px-4 py-1">{{ ticket.title }}</td>
                <td class="px-4 py-1 whitespace-nowrap">
                  <StatusBadge type="status" :value="ticket.status" />
                </td>
                <td class="px-4 py-1 whitespace-nowrap">
                  <StatusBadge type="priority" :value="ticket.priority" :short="true" />
                </td>
                <td class="px-4 py-1 whitespace-nowrap">{{ ticket.created }}</td>
                <td class="px-4 py-1 whitespace-nowrap">
                  <UserAvatar :name="ticket.assignee" />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="tickets.length === 0" class="p-8 text-center text-gray-400">
        No tickets found. Create a new ticket to get started.
      </div>
    </div>
  </div>
</template>