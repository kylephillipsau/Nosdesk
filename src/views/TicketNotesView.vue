<!-- TicketNotesView.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TicketArticleBody from '@/components/ticketComponents/TicketArticleBody.vue'

const route = useRoute()
const router = useRouter()
const ticketContent = ref('')
const ticketTitle = ref('')

const fetchTicket = async (ticketId: string | string[]) => {
  try {
    // TODO: Replace with actual API call
    const ticketData = (await import("@/assets/tickets.json")).default
    const ticket = ticketData.tickets.find((t: any) => t.id === Number(ticketId))
    if (!ticket) {
      router.push('/error/404')
      return
    }
    ticketContent.value = ticket.articleContent || ''
    ticketTitle.value = ticket.title || ''
  } catch (error) {
    console.error('Failed to fetch ticket:', error)
    // TODO: Add error handling
  }
}

const handleBack = () => {
  router.push(`/tickets/${route.params.id}`)
}

onMounted(() => {
  if (route.params.id) {
    fetchTicket(route.params.id)
  }
})
</script>

<template>
  <div class="flex-1 flex flex-col h-full bg-slate-900">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 bg-slate-800 border-b border-slate-700">
      <div class="flex items-center gap-4">
        <button
          @click="handleBack"
          class="p-2 text-slate-400 hover:text-white hover:bg-slate-700 rounded-lg transition-colors"
          title="Back to ticket"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
          </svg>
        </button>
        <div>
          <h1 class="text-lg font-medium text-white">
            {{ ticketTitle }}
          </h1>
          <p class="text-sm text-slate-400">Ticket #{{ route.params.id }}</p>
        </div>
      </div>
    </div>

    <!-- Editor Container -->
    <div class="flex-1 p-6">
      <TicketArticleBody
        :initial-content="ticketContent"
        v-model:content="ticketContent"
        class="h-full"
      />
    </div>
  </div>
</template>

<style scoped>
:deep(.bg-slate-800) {
  background-color: transparent;
}

:deep(.rounded-2xl) {
  border-radius: 0;
}

:deep(.shadow-lg) {
  box-shadow: none;
}

:deep(.editor-wrapper) {
  height: calc(100vh - 12rem);
}

:deep(.editor-container) {
  height: calc(100% - 3rem);
}
</style> 