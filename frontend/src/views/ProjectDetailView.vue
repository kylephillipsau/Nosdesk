<!-- ProjectDetailView.vue -->
<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import { projectService } from '@/services/projectService'
import Modal from '@/components/Modal.vue'
import ProjectForm from '@/components/projectComponents/ProjectForm.vue'
import TicketSelectionModal from '@/components/projectComponents/TicketSelectionModal.vue'
import KanbanBoard from '@/components/projectComponents/KanbanBoard.vue'
import StatusBadge from '@/components/StatusBadge.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import BackButton from '@/components/common/BackButton.vue'

const route = useRoute()
const router = useRouter()
const projectId = computed(() => Number(route.params.id))

const project = ref<Project | null>(null)
const tickets = ref<any[]>([])
const isLoading = ref(true)
const isTicketsLoading = ref(false)
const error = ref<string | null>(null)
const showEditModal = ref(false)
const showAddTicketModal = ref(false)
const activeTab = computed(() => {
  return route.query.view === 'list' ? 'list' : 'kanban'
})

// Fetch project details on component mount
onMounted(async () => {
  if (!projectId.value) {
    router.push('/projects')
    return
  }
  
  try {
    isLoading.value = true
    error.value = null
    
    // Fetch project details
    project.value = await projectService.getProject(projectId.value)
    
    // Fetch project tickets
    await fetchProjectTickets()
  } catch (err) {
    console.error('Failed to fetch project details:', err)
    error.value = 'Failed to load project details. Please try again later.'
  } finally {
    isLoading.value = false
  }
})

const fetchProjectTickets = async () => {
  if (!projectId.value) return
  
  try {
    isTicketsLoading.value = true
    tickets.value = await projectService.getProjectTickets(projectId.value)
  } catch (err) {
    console.error('Failed to fetch project tickets:', err)
    error.value = 'Failed to load project tickets. Please try again later.'
  } finally {
    isTicketsLoading.value = false
  }
}

const handleEditProject = async (projectData: Omit<Project, 'id' | 'ticketCount'> & { id?: number }) => {
  if (!project.value) return
  
  try {
    isLoading.value = true
    error.value = null
    
    const updatedProject = await projectService.updateProject(
      project.value.id,
      projectData
    )
    
    project.value = updatedProject
    showEditModal.value = false
  } catch (err) {
    console.error('Failed to edit project:', err)
    error.value = 'Failed to update project. Please try again.'
  } finally {
    isLoading.value = false
  }
}

const handleDeleteProject = async () => {
  if (!project.value) return
  
  if (!confirm(`Are you sure you want to delete the project "${project.value.name}"?`)) {
    return
  }
  
  try {
    isLoading.value = true
    error.value = null
    
    await projectService.deleteProject(project.value.id)
    router.push('/projects')
  } catch (err) {
    console.error('Failed to delete project:', err)
    error.value = 'Failed to delete project. Please try again.'
  } finally {
    isLoading.value = false
  }
}

const handleAddTicket = async (ticketId: number) => {
  if (!project.value) return
  
  try {
    error.value = null
    
    await projectService.addTicketToProject(project.value.id, ticketId)
    
    // Refresh the ticket list
    await fetchProjectTickets()
    
    // Update the project to get the new ticket count
    project.value = await projectService.getProject(project.value.id)
    
    showAddTicketModal.value = false
  } catch (err) {
    console.error('Failed to add ticket to project:', err)
    error.value = 'Failed to add ticket to project. Please try again.'
  }
}

const handleRemoveTicket = async (ticketId: number) => {
  if (!project.value) return
  
  if (!confirm('Are you sure you want to remove this ticket from the project?')) {
    return
  }
  
  try {
    error.value = null
    
    await projectService.removeTicketFromProject(project.value.id, ticketId)
    
    // Refresh the ticket list
    await fetchProjectTickets()
    
    // Update the project to get the new ticket count
    project.value = await projectService.getProject(project.value.id)
  } catch (err) {
    console.error('Failed to remove ticket from project:', err)
    error.value = 'Failed to remove ticket from project. Please try again.'
  }
}

const goToTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

const getStatusClass = (status: string) => {
  switch (status) {
    case 'active':
      return 'bg-green-500/20 text-green-400 border-green-500/30'
    case 'completed':
      return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    case 'archived':
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    default:
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
  }
}

const getPriorityClass = (priority: string) => {
  switch (priority) {
    case 'high':
      return 'bg-red-500/20 text-red-400 border-red-500/30'
    case 'medium':
      return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
    case 'low':
      return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    default:
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
  }
}

// Get existing ticket IDs for filtering in the ticket selection modal
const existingTicketIds = computed(() => tickets.value.map(ticket => ticket.id))

// Update URL when view changes
const setActiveTab = (tab: string) => {
  router.replace({ 
    query: { 
      ...route.query,
      view: tab === 'kanban' ? undefined : tab 
    } 
  })
}

// Watch for route query changes to sync the active tab
watch(() => route.query.view, (newValue) => {
  // No need to set activeTab since it's now computed from the route
  console.log(`View changed to ${newValue || 'kanban'}`)
}, { immediate: true })
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex flex-col gap-4 p-6">
      <!-- Back button -->
      <BackButton fallbackRoute="/projects" label="Back to Projects" />

      <!-- Error message -->
      <div v-if="error" class="bg-red-500/20 border border-red-500/50 text-red-200 px-4 py-3 rounded-lg mb-4">
        {{ error }}
      </div>

      <!-- Loading state -->
      <div v-if="isLoading" class="flex justify-center items-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>

      <!-- Project details -->
      <div v-else-if="project" class="flex flex-col gap-4">
        <!-- Project header -->
        <div class="flex justify-between items-start">
          <div>
            <h1 class="text-2xl font-semibold text-white">{{ project.name }}</h1>
            <p class="text-slate-400 mt-2">{{ project.description }}</p>
          </div>
          <div class="flex items-center gap-2">
            <span 
              :class="getStatusClass(project.status)"
              class="px-3 py-1 rounded-full text-sm border"
            >
              {{ project.status }}
            </span>
            <button 
              @click="showEditModal = true"
              class="p-2 text-slate-400 hover:text-white transition-colors"
              title="Edit project"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
              </svg>
            </button>
            <button 
              @click="handleDeleteProject"
              class="p-2 text-slate-400 hover:text-red-400 transition-colors"
              title="Delete project"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Project view tabs -->
        <div class="border-b border-slate-700">
          <div class="flex space-x-4">
            <button 
              @click="setActiveTab('kanban')"
              class="py-2 px-4 border-b-2 font-medium text-sm"
              :class="activeTab === 'kanban' ? 'border-blue-500 text-blue-500' : 'border-transparent text-slate-400 hover:text-slate-300'"
            >
              Kanban Board
            </button>
            <button 
              @click="setActiveTab('list')"
              class="py-2 px-4 border-b-2 font-medium text-sm"
              :class="activeTab === 'list' ? 'border-blue-500 text-blue-500' : 'border-transparent text-slate-400 hover:text-slate-300'"
            >
              List View
            </button>
          </div>
        </div>

        <!-- Kanban Board View -->
        <div v-if="activeTab === 'kanban'" class="flex-1 min-h-[500px]">
          <KanbanBoard :project-id="project.id" />
        </div>

        <!-- List View -->
        <div v-else class="flex flex-col gap-4">
          <div class="flex justify-between items-center mb-4">
            <h2 class="text-xl font-medium text-white">Tickets</h2>
            <button 
              @click="showAddTicketModal = true"
              class="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
            >
              Add Ticket
            </button>
          </div>
          
          <div v-if="isTicketsLoading" class="flex justify-center items-center py-8">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
          </div>
          
          <div v-else-if="tickets.length === 0" class="text-center py-8 bg-slate-800 rounded-lg">
            <p class="text-slate-400">No tickets associated with this project.</p>
            <button 
              @click="showAddTicketModal = true"
              class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Add Tickets
            </button>
          </div>
          
          <div v-else class="bg-slate-800 rounded-lg overflow-hidden">
            <table class="w-full">
              <thead>
                <tr class="border-b border-slate-700">
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">ID</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Title</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Status</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Priority</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Assignee</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-700">
                <tr 
                  v-for="ticket in tickets" 
                  :key="ticket.id"
                  class="hover:bg-slate-700 transition-colors"
                >
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-slate-300">#{{ ticket.id }}</td>
                  <td 
                    @click="goToTicket(ticket.id)"
                    class="px-6 py-4 whitespace-nowrap text-sm text-white cursor-pointer hover:underline"
                  >
                    {{ ticket.title }}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <StatusBadge 
                      type="status" 
                      :value="ticket.status"
                    />
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <StatusBadge 
                      type="priority" 
                      :value="ticket.priority"
                      short
                    />
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <UserAvatar 
                      v-if="ticket.assignee" 
                      :name="ticket.assignee" 
                      size="xs" 
                      :showName="true"
                      :clickable="false"
                    />
                    <span v-else class="text-xs text-slate-400">Unassigned</span>
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <button 
                      @click="handleRemoveTicket(ticket.id)"
                      class="text-red-400 hover:text-red-300 transition-colors"
                      title="Remove from project"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                      </svg>
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Project Modal -->
    <Modal
      :show="showEditModal"
      title="Edit Project"
      @close="showEditModal = false"
    >
      <ProjectForm
        v-if="project"
        mode="edit"
        :project="project"
        :disabled="isLoading"
        @submit="handleEditProject"
        @cancel="showEditModal = false"
      />
    </Modal>

    <!-- Add Ticket Modal -->
    <TicketSelectionModal
      v-if="project"
      :show="showAddTicketModal"
      :project-id="project.id"
      :existing-ticket-ids="existingTicketIds"
      @close="showAddTicketModal = false"
      @select-ticket="handleAddTicket"
    />
  </div>
</template> 