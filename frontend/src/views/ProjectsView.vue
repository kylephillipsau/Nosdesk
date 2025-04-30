<!-- ProjectsView.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import Modal from '@/components/Modal.vue'
import ProjectForm from '@/components/projectComponents/ProjectForm.vue'
import { projectService } from '@/services/projectService'

const projects = ref<Project[]>([])
const router = useRouter()
const showCreateModal = ref(false)
const showEditModal = ref(false)
const isLoading = ref(true)
const error = ref<string | null>(null)
const selectedProject = ref<Project | null>(null)

// Fetch projects on component mount
onMounted(async () => {
  try {
    isLoading.value = true
    projects.value = await projectService.getProjects()
  } catch (err) {
    console.error('Failed to fetch projects:', err)
    error.value = 'Failed to load projects. Please try again later.'
  } finally {
    isLoading.value = false
  }
})

const openProject = (projectId: number) => {
  router.push(`/projects/${projectId}`)
}

const handleEditProject = async (projectData: Omit<Project, 'id' | 'ticketCount'> & { id?: number }) => {
  if (!selectedProject.value?.id) return
  
  try {
    isLoading.value = true
    error.value = null
    
    const updatedProject = await projectService.updateProject(
      selectedProject.value.id,
      projectData
    )
    
    // Update the project in the local array
    const index = projects.value.findIndex(p => p.id === selectedProject.value?.id)
    if (index !== -1) {
      projects.value[index] = updatedProject
    }
    
    showEditModal.value = false
    selectedProject.value = null
  } catch (err) {
    console.error('Failed to edit project:', err)
    error.value = 'Failed to update project. Please try again.'
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
    error.value = null
    
    const newProject = await projectService.createProject(projectData)
    projects.value.push(newProject)
    
    showCreateModal.value = false
  } catch (err) {
    console.error('Failed to create project:', err)
    error.value = 'Failed to create project. Please try again.'
  } finally {
    isLoading.value = false
  }
}

const removeProject = async (event: Event, projectId: number) => {
  event.stopPropagation()
  if (!confirm('Are you sure you want to remove this project?')) return

  try {
    isLoading.value = true
    error.value = null
    
    await projectService.deleteProject(projectId)
    projects.value = projects.value.filter(p => p.id !== projectId)
  } catch (err) {
    console.error('Failed to remove project:', err)
    error.value = 'Failed to delete project. Please try again.'
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

      <!-- Error message -->
      <div v-if="error" class="bg-red-500/20 border border-red-500/50 text-red-200 px-4 py-3 rounded-lg mb-4">
        {{ error }}
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
          @click="openProject(project.id)" 
          :key="project.id"
          class="bg-slate-800 rounded-lg p-4 group relative hover:bg-slate-700 transition-colors cursor-pointerno"
        >
          <div class="flex flex-col gap-2">
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
          
          <!-- Action buttons -->
          <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button 
              @click="(e) => openEditModal(e, project)" 
              class="p-1 text-slate-400 hover:text-white"
              title="Edit project"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
              </svg>
            </button>
            <button 
              @click="(e) => removeProject(e, project.id)" 
              class="p-1 text-slate-400 hover:text-red-400"
              title="Delete project"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            </button>
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