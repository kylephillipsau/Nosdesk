<!-- ProjectSelectionModal.vue -->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import Modal from '@/components/Modal.vue'
import type { Project } from '@/types/project'
import { projectService } from '@/services/projectService'

const props = defineProps<{
  show: boolean;
  currentProjectId?: number;
}>()

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select-project', projectId: number): void;
}>()

const projects = ref<Project[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')

// Fetch projects when the modal is shown
watch(() => props.show, async (isVisible) => {
  if (isVisible) {
    await fetchProjects()
  }
})

// Fetch projects on component mount
onMounted(async () => {
  if (props.show) {
    await fetchProjects()
  }
})

const fetchProjects = async () => {
  isLoading.value = true
  error.value = null
  
  try {
    projects.value = await projectService.getProjects()
  } catch (err) {
    console.error('Failed to fetch projects:', err)
    error.value = 'Failed to load projects. Please try again later.'
  } finally {
    isLoading.value = false
  }
}

const filteredProjects = computed(() => {
  const query = searchQuery.value.toLowerCase()
  return projects.value.filter(project => 
    project.name.toLowerCase().includes(query) ||
    project.description?.toLowerCase().includes(query)
  )
})

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

const selectProject = (projectId: number) => {
  emit('select-project', projectId)
}
</script>

<template>
  <Modal
    :show="show"
    title="Add to Project"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-4 px-2">
      <!-- Search -->
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search projects..."
          class="w-full pl-10 pr-4 py-2 bg-slate-700 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <svg class="h-5 w-5 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
      </div>

      <!-- Error message -->
      <div v-if="error" class="bg-red-500/20 border border-red-500/50 text-red-200 px-4 py-3 rounded-lg">
        {{ error }}
      </div>

      <!-- Loading state -->
      <div v-if="isLoading" class="flex justify-center items-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>

      <!-- Project List -->
      <div v-else class="flex flex-col gap-2 max-h-[400px] overflow-y-auto p-2">
        <div
          v-for="project in filteredProjects"
          :key="project.id"
          @click="selectProject(project.id)"
          class="flex flex-col gap-1 p-3 bg-slate-700 rounded-lg cursor-pointer hover:bg-slate-600 transition-colors"
          :class="{ 'ring-2 ring-blue-500': project.id === currentProjectId }"
        >
          <div class="flex items-center justify-between">
            <h4 class="font-medium text-white">{{ project.name }}</h4>
            <span 
              :class="getStatusClass(project.status)"
              class="text-sm px-2 py-0.5 rounded-full border"
            >
              {{ project.status }}
            </span>
          </div>
          <p v-if="project.description" class="text-sm text-slate-400">
            {{ project.description }}
          </p>
          <div class="text-xs text-slate-400 mt-1">
            {{ project.ticketCount }} tickets
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div
        v-if="!isLoading && filteredProjects.length === 0"
        class="text-center py-8 text-slate-400"
      >
        No projects found
      </div>
    </div>
  </Modal>
</template> 