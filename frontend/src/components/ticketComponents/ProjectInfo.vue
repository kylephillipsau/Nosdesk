<!-- components/ProjectInfo.vue -->
<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Project } from '@/services/ticketService';

const props = defineProps<{
  project: Project;
  projectId: string;
}>();

const emit = defineEmits<{
  (e: 'remove'): void;
  (e: 'view'): void;
}>();

// Compute ticket count from project data or use a default value
const ticketCount = computed(() => {
  // Check if the project has a ticket_count property (from API)
  if ('ticket_count' in props.project) {
    return (props.project as any).ticket_count;
  }
  return '—';
});

const getStatusClass = (status: string) => {
  switch (status) {
    case 'active':
      return 'bg-green-500/20 text-green-400 border-green-500/30';
    case 'completed':
      return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
    case 'archived':
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    default:
      return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  }
};
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 overflow-hidden hover:border-slate-600/50 transition-colors">
    <!-- Header -->
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-medium text-white">{{ project.name }}</h3>
        
        <!-- Action buttons -->
        <div class="flex items-center gap-1">
          <button
            @click="emit('view')"
            class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded-md transition-colors"
            title="View project"
          >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
              <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
            </svg>
          </button>
          <button
            @click="emit('remove')"
            class="p-1.5 text-slate-400 hover:text-red-400 hover:bg-red-900/20 rounded-md transition-colors"
            title="Remove from project"
          >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Content -->
    <div class="p-4">
      <div class="flex flex-col gap-3">
        <!-- Description -->
        <div v-if="project.description" class="flex flex-col gap-1">
          <span class="text-xs text-slate-400 uppercase tracking-wide">Description</span>
          <p class="text-sm text-slate-200">{{ project.description }}</p>
        </div>
        
        <!-- Project metadata -->
        <div class="grid grid-cols-3 gap-3 text-sm">
          <div class="flex flex-col gap-1">
            <span class="text-xs text-slate-400 uppercase tracking-wide">Project ID</span>
            <span class="text-slate-200 font-mono text-xs">#{{ projectId }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-slate-400 uppercase tracking-wide">Status</span>
            <span 
              :class="getStatusClass(project.status)"
              class="text-xs px-2 py-1 rounded-md border w-fit"
            >
              {{ project.status }}
            </span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-slate-400 uppercase tracking-wide">Tickets</span>
            <span class="text-slate-200 text-xs">{{ ticketCount }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>