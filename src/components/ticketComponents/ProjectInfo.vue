<!-- components/ProjectInfo.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import type { Project } from '@/types/project';

const props = defineProps<{
  project: Project;
  projectId: string;
}>();

const emit = defineEmits<{
  (e: 'remove'): void;
  (e: 'view'): void;
}>();

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'text-green-400';
    case 'completed':
      return 'text-blue-400';
    case 'archived':
      return 'text-gray-400';
    default:
      return 'text-slate-400';
  }
};
</script>

<template>
  <div class="bg-slate-800 rounded-lg overflow-hidden">
    <div class="px-4 py-3 bg-slate-700/50 flex items-center justify-between">
      <h2 class="text-lg font-medium text-slate-100">{{ project.name }}</h2>

      <div class="flex items-center gap-2">
        <button
          @click="emit('view')"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
          title="View project"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </button>
        <button
          @click="emit('remove')"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
          title="Remove from project"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
    </div>
    <div class="p-4">
      <div class="flex flex-col gap-2">
        <div class="flex-1">
          <p class="text-sm text-slate-400 mt-0.5 line-clamp-2">{{ project.description }}</p>
        </div>
        <div class="flex items-center gap-2 mt-2">
          <span class="text-xs px-2 py-0.5 bg-slate-600/50 text-slate-300 rounded">
            #{{ projectId }}
          </span>
          <span :class="[getStatusColor(project.status), 'text-xs']">
            {{ project.status }}
          </span>
          <span class="text-xs text-slate-400">
            {{ project.ticketCount }} tickets
          </span>
        </div>
      </div>
    </div>
  </div>
</template>