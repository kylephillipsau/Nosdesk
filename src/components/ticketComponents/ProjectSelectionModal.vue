<!-- ProjectSelectionModal.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import Modal from '@/components/Modal.vue'
import type { Project } from '@/types/project'

const props = defineProps<{
  show: boolean;
  currentProjectId?: number;
}>()

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select-project', projectId: number): void;
}>()

// TODO: Replace with actual API call or store
const projects = ref<Project[]>([
  {
    id: 1,
    name: "Website Redesign",
    description: "Complete overhaul of the company website",
    status: 'active',
    ticketCount: 5
  },
  {
    id: 2,
    name: "Mobile App Development",
    description: "New mobile app for customer support",
    status: 'active',
    ticketCount: 8
  },
  {
    id: 3,
    name: "Infrastructure Upgrade",
    description: "Upgrade server infrastructure and monitoring",
    status: 'completed',
    ticketCount: 3
  }
])

const searchQuery = ref('')

const filteredProjects = computed(() => {
  const query = searchQuery.value.toLowerCase()
  return projects.value.filter(project => 
    project.name.toLowerCase().includes(query) ||
    project.description?.toLowerCase().includes(query)
  )
})

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

      <!-- Project List -->
      <div class="flex flex-col gap-2 max-h-[400px] overflow-y-auto p-2">
        <div
          v-for="project in filteredProjects"
          :key="project.id"
          @click="selectProject(project.id)"
          class="flex flex-col gap-1 p-3 bg-slate-700 rounded-lg cursor-pointer hover:bg-slate-600 transition-colors"
          :class="{ 'ring-2 ring-blue-500': project.id === currentProjectId }"
        >
          <div class="flex items-center justify-between">
            <h4 class="font-medium text-white">{{ project.name }}</h4>
            <span :class="[getStatusColor(project.status), 'text-sm px-2 py-0.5 rounded-full bg-slate-600/50']">
              {{ project.status }}
            </span>
          </div>
          <p v-if="project.description" class="text-sm text-slate-400">
            {{ project.description }}
          </p>
        </div>
      </div>

      <!-- Empty State -->
      <div
        v-if="filteredProjects.length === 0"
        class="text-center py-8 text-slate-400"
      >
        No projects found
      </div>
    </div>
  </Modal>
</template> 