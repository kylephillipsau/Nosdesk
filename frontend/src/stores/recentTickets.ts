import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  requester?: string;
  assignee?: string;
  created?: string;
  modified?: string;
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

    console.log(`Adding/updating ticket #${ticket.id} with title: "${ticket.title}" to recent tickets store`)

    // Add isDraft flag if specified
    const ticketWithDraftStatus = {
      ...ticket,
      isDraft: isDraft
    }

    // Check if ticket already exists
    const existingIndex = recentTickets.value.findIndex(t => t.id === ticket.id)
    
    if (existingIndex !== -1) {
      console.log(`Ticket #${ticket.id} already exists in recent tickets store with title: "${recentTickets.value[existingIndex].title}"`)
      
      // Preserve the existing title unless explicitly changing it
      // This ensures manual title changes aren't overwritten during refreshes
      if (ticket.title !== 'New Ticket') {
        const existingTitle = recentTickets.value[existingIndex].title
        // Only preserve the title if it's been manually changed (not the default or same as incoming)
        if (existingTitle !== 'New Ticket' && existingTitle !== ticket.title) {
          console.log(`Preserving existing title: "${existingTitle}" instead of using: "${ticket.title}"`)
          ticketWithDraftStatus.title = existingTitle
        }
      }
      
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
    
    // Force reactivity update
    recentTickets.value = [...recentTickets.value]
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

  // Enhanced method to update ticket data when it's modified in TicketView
  const updateTicketData = (ticketId: number, updatedData: Partial<Ticket>) => {
    console.log(`Attempting to update ticket #${ticketId} with data:`, updatedData)
    
    const ticketIndex = recentTickets.value.findIndex(t => t.id === ticketId)
    console.log(`Found ticket at index: ${ticketIndex}`)
    
    if (ticketIndex !== -1) {
      const oldData = { ...recentTickets.value[ticketIndex] }
      
      // Update only the provided fields, preserving other data
      recentTickets.value[ticketIndex] = {
        ...recentTickets.value[ticketIndex],
        ...updatedData
      }
      
      console.log(`Updated ticket #${ticketId} in recent tickets store:`)
      console.log(`- Before:`, oldData)
      console.log(`- After:`, recentTickets.value[ticketIndex])
      console.log(`- Changes:`, updatedData)
      
      // Force reactivity update by creating a new array
      recentTickets.value = [...recentTickets.value]
    } else {
      console.warn(`Ticket #${ticketId} not found in recent tickets store`)
    }
  }

  return {
    recentTickets,
    addRecentTicket,
    removeRecentTicket,
    updateDraftStatus,
    updateTicketData
  }
})