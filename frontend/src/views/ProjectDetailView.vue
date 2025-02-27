<!-- ProjectDetailView.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import KanbanBoard from '@/components/projectComponents/KanbanBoard.vue'
import Modal from '@/components/Modal.vue'
import ProjectForm from '@/components/projectComponents/ProjectForm.vue'

const route = useRoute()
const router = useRouter()

const project = ref<Project | null>(null)
const isLoading = ref(true)
const showEditModal = ref(false)

const fetchProject = async (projectId: string | string[]) => {
  try {
    isLoading.value = true
    // TODO: Replace with actual API call
    const mockProject: Project = {
      id: Number(projectId),
      name: "Website Redesign",
      description: "Complete overhaul of the company website",
      ticketCount: 5,
      status: 'active'
    }

    project.value = mockProject
  } catch (error) {
    console.error('Failed to fetch project:', error)
    // TODO: Add error handling
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  if (route.params.id) {
    fetchProject(route.params.id)
  }
})

const handleEditProject = async (projectData: Omit<Project, 'id' | 'ticketCount'>) => {
  if (!project.value) return

  try {
    isLoading.value = true
    // TODO: Replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 500))

    project.value = {
      ...project.value,
      ...projectData
    }
    showEditModal.value = false
  } catch (error) {
    console.error('Failed to update project:', error)
    // TODO: Add error handling
  } finally {
    isLoading.value = false
  }
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'text-green-400'
    case 'completed':
      return 'text-blue-400'
    case 'archived':
      return 'text-gray-400'
    default:
      return 'text-slate-400'
  }
}
</script>

<template>
  <div v-if="isLoading" class="flex-1 flex items-center justify-center">
    <div class="text-slate-400">Loading project...</div>
  </div>

  <div v-else-if="project" class="flex-1 flex flex-col h-full">
    <!-- Project Header -->
    <div class="bg-slate-800 border-b border-slate-700 flex-shrink-0">
      <div class="flex flex-col gap-2 px-6 py-4">
        <div class="flex items-start justify-between">
          <div>
            <h1 class="text-2xl font-semibold text-white">{{ project.name }}</h1>
            <p class="mt-1 text-slate-400">{{ project.description }}</p>
          </div>
          <div class="flex items-center gap-2">
            <span :class="[getStatusColor(project.status), 'text-sm']">
              {{ project.status }}
            </span>
            <button
              @click="showEditModal = true"
              class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              :disabled="isLoading"
            >
              Edit Project
            </button>
          </div>
        </div>

        <!-- Project Stats -->
        <div class="mt-4 flex gap-4">
          <div class="px-4 py-2 bg-slate-700/50 rounded-lg">
            <span class="text-sm text-slate-400">Total Tickets</span>
            <div class="text-lg font-medium text-white">{{ project.ticketCount }}</div>
          </div>
          <!-- Add more stats as needed -->
        </div>
      </div>
    </div>

    <!-- Kanban Board Container -->
    <div class="flex-1 min-h-0">
      <KanbanBoard :project-id="project.id" />
    </div>

    <!-- Edit Project Modal -->
    <Modal
      :show="showEditModal"
      title="Edit Project"
      @close="showEditModal = false"
    >
      <ProjectForm
        v-if="showEditModal"
        mode="edit"
        :project="project"
        :disabled="isLoading"
        @submit="handleEditProject"
        @cancel="showEditModal = false"
      />
    </Modal>
  </div>

  <div v-else class="flex-1 flex items-center justify-center">
    <div class="text-slate-400">Project not found</div>
  </div>
</template> 