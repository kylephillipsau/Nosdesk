<script setup lang="ts">
import ticketData from '@/assets/tickets.json'
import { ref } from 'vue'
import UserAvatar from '@/components/UserAvatar.vue'

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  priority: 'low' | 'medium' | 'high';
  created: string;
  assignee: string;
}

const tickets: Ticket[] = ticketData.tickets
const selectedTickets = ref<number[]>([])

const toggleSelection = (event: Event, ticketId: number) => {
  // Stop propagation to prevent row click when checking checkbox
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

import { useRouter } from 'vue-router'

const router = useRouter()

const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

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
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex justify-between items-center p-6 border-b border-gray-800">
      <h1 class="text-2xl font-semibold text-white">Tickets List</h1>
      <button class="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg transition-colors text-white">
        New Ticket
      </button>
    </div>

    <!-- Table Container -->
    <div class="flex-1 overflow-auto">
      <table class="w-full text-sm text-gray-200">
        <thead class="bg-gray-800/50">
          <tr>
            <th class="p-4 w-10">
              <input type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedTickets.length === tickets.length" @change="toggleAllTickets">
            </th>
            <th class="text-left p-4 font-medium w-20">ID</th>
            <th class="text-left p-4 font-medium">Title</th>
            <th class="text-left p-4 font-medium w-24">Status</th>
            <th class="text-left p-4 font-medium w-24">Priority</th>
            <th class="text-left p-4 font-medium w-32">Created</th>
            <th class="text-left p-4 font-medium w-32">Assignee</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="ticket in tickets" :key="ticket.id"
            class="border-b border-gray-800 hover:bg-gray-800/50 transition-colors cursor-pointer"
            @click="openTicket(ticket.id)">
            <td class="px-4 py-1">
              <input type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedTickets.includes(ticket.id)" @change="(e) => toggleSelection(e, ticket.id)">
            </td>
            <td class="px-4 py-1">#{{ ticket.id }}</td>
            <td class="px-4 py-1">{{ ticket.title }}</td>
            <td class="px-4 py-1">
              <span :class="[getStatusColor(ticket.status), 'px-2 py-1 rounded-full text-xs inline-block whitespace-nowrap']">
                {{ ticket.status }}
              </span>
            </td>
            <td class="px-4 py-1">
              <span :class="[getPriorityColor(ticket.priority), 'font-medium']">
                {{ ticket.priority }}
              </span>
            </td>
            <td class="px-4 py-1">{{ ticket.created }}</td>
            <td class="px-4 py-1">
              <UserAvatar :name="ticket.assignee" />
            </td>
          </tr>
        </tbody>
      </table>

      <!-- Empty state -->
      <div v-if="tickets.length === 0" class="p-8 text-center text-gray-400">
        No tickets found. Create a new ticket to get started.
      </div>
    </div>
  </div>
</template>