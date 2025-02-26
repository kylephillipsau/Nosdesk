import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  requester?: string;
  assignee?: string;
  created?: string;
  isDraft?: boolean;
}

export const useRecentTicketsStore = defineStore('recentTickets', () => {
  const recentTickets = ref<Ticket[]>([])
  const MAX_RECENT_TICKETS = 15

  // Initialize from localStorage if available
  if (localStorage.getItem('recentTickets')) {
    recentTickets.value = JSON.parse(localStorage.getItem('recentTickets')!)
  }

  // Save to localStorage when updated
  watch(recentTickets, (newTickets) => {
    localStorage.setItem('recentTickets', JSON.stringify(newTickets))
  }, { deep: true })

  const addRecentTicket = (ticket: Ticket, fromRecentList: boolean = false, isDraft: boolean = false) => {
    // If clicking from recent list, don't rearrange
    if (fromRecentList) {
      return
    }

    // Add isDraft flag if specified
    const ticketWithDraftStatus = {
      ...ticket,
      isDraft: isDraft
    }

    // Check if ticket already exists
    const existingIndex = recentTickets.value.findIndex(t => t.id === ticket.id)
    
    if (existingIndex !== -1) {
      // Update existing ticket but preserve isDraft status if not explicitly changing it
      if (isDraft === false && recentTickets.value[existingIndex].isDraft === true) {
        ticketWithDraftStatus.isDraft = recentTickets.value[existingIndex].isDraft
      }
      // Remove from current position
      recentTickets.value.splice(existingIndex, 1)
    }
    
    // Add to start of array
    recentTickets.value.unshift(ticketWithDraftStatus)
    
    // Keep only the most recent tickets
    if (recentTickets.value.length > MAX_RECENT_TICKETS) {
      recentTickets.value = recentTickets.value.slice(0, MAX_RECENT_TICKETS)
    }
  }

  const removeRecentTicket = (ticketId: number) => {
    recentTickets.value = recentTickets.value.filter(t => t.id !== ticketId)
  }

  const updateDraftStatus = (ticketId: number, isDraft: boolean) => {
    const ticketIndex = recentTickets.value.findIndex(t => t.id === ticketId)
    if (ticketIndex !== -1) {
      recentTickets.value[ticketIndex].isDraft = isDraft
    }
  }

  return {
    recentTickets,
    addRecentTicket,
    removeRecentTicket,
    updateDraftStatus
  }
})