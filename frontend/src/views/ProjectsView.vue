<!-- ProjectsView.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import Modal from '@/components/Modal.vue'
import ProjectForm from '@/components/projectComponents/ProjectForm.vue'

const projects = ref<Project[]>([
  {
    id: 1,
    name: "Website Redesign",
    description: "Complete overhaul of the company website",
    ticketCount: 5,
    status: 'active'
  },
  {
    id: 2,
    name: "Mobile App Development",
    description: "New mobile app for customer support",
    ticketCount: 8,
    status: 'active'
  },
  {
    id: 3,
    name: "Infrastructure Upgrade",
    description: "Upgrade server infrastructure and monitoring",
    ticketCount: 3,
    status: 'completed'
  }
])

const router = useRouter()
const showCreateModal = ref(false)
const showEditModal = ref(false)
const isLoading = ref(false)
const selectedProject = ref<Project | null>(null)

const openProject = (projectId: number) => {
  router.push(`/projects/${projectId}`)
}

const handleEditProject = async (projectData: Omit<Project, 'id' | 'ticketCount'> & { id?: number }) => {
  try {
    isLoading.value = true
    // TODO: Replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 500))
    
    if (selectedProject.value?.id) {
      const index = projects.value.findIndex(p => p.id === selectedProject.value?.id)
      if (index !== -1) {
        projects.value[index] = {
          ...projects.value[index],
          ...projectData,
          id: selectedProject.value.id,
        }
      }
    }
    showEditModal.value = false
    selectedProject.value = null
  } catch (error) {
    console.error('Failed to edit project:', error)
    // TODO: Add error handling
  } finally {
    isLoading.value = false
  }
}

const openEditModal = (event: Event, project: Project) => {
  event.stopPropagation()
  selectedProject.value = { ...project }
  showEditModal.value = true
}

const handleCreateProject = async (projectData: Omit<Project, 'id' | 'ticketCount'>) => {
  try {
    isLoading.value = true
    // TODO: Replace with actual API call
    const newProject: Project = {
      id: Math.max(...projects.value.map(p => p.id)) + 1,
      ticketCount: 0,
      ...projectData
    }

    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 500))
    projects.value.push(newProject)
    showCreateModal.value = false
  } catch (error) {
    console.error('Failed to create project:', error)
    // TODO: Add error handling
  } finally {
    isLoading.value = false
  }
}

const removeProject = async (event: Event, projectId: number) => {
  event.stopPropagation()
  if (!confirm('Are you sure you want to remove this project?')) return

  try {
    isLoading.value = true
    // TODO: Replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 500))
    projects.value = projects.value.filter(p => p.id !== projectId)
  } catch (error) {
    console.error('Failed to remove project:', error)
    // TODO: Add error handling
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="flex flex-col gap-4 p-6">
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-semibold text-white">Projects</h1>
        <button
          @click="showCreateModal = true"
          class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          :disabled="isLoading"
        >
          Create Project
        </button>
      </div>

      <div v-if="isLoading" class="flex justify-center items-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>

      <div v-else-if="projects.length === 0" class="text-center py-8">
        <p class="text-slate-400">No projects found. Create your first project to get started.</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="project in projects"
          :key="project.id"
          class="bg-slate-800 rounded-lg p-6 group relative hover:bg-slate-700 transition-colors"
        >
          <div @click="openProject(project.id)" class="flex flex-col gap-2 cursor-pointer">
            <div class="flex items-start justify-between">
              <h3 class="text-lg font-medium text-white">{{ project.name }}</h3>
              <span
                :class="{
                  'text-green-400': project.status === 'active',
                  'text-blue-400': project.status === 'completed',
                  'text-gray-400': project.status === 'archived'
                }"
                class="text-sm px-2 py-1 rounded-full bg-slate-700/50"
              >
                {{ project.status }}
              </span>
            </div>
            <p class="text-slate-400 text-sm mt-3 line-clamp-2">{{ project.description }}</p>
            <div class="mt-6 flex items-center gap-4 text-sm text-slate-400">
              <span class="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd" />
                </svg>
                {{ project.ticketCount }} tickets
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Project Modal -->
    <Modal
      :show="showCreateModal"
      title="Create New Project"
      @close="showCreateModal = false"
    >
      <ProjectForm
        mode="create"
        :disabled="isLoading"
        @submit="handleCreateProject"
        @cancel="showCreateModal = false"
      />
    </Modal>

    <!-- Edit Project Modal -->
    <Modal
      :show="showEditModal"
      title="Edit Project"
      @close="showEditModal = false"
    >
      <ProjectForm
        v-if="selectedProject"
        mode="edit"
        :project="selectedProject"
        :disabled="isLoading"
        @submit="handleEditProject"
        @cancel="showEditModal = false"
      />
    </Modal>
  </div>
</template> 