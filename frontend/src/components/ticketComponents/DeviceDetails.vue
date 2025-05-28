<!-- components/DeviceDetails.vue -->
<script setup lang="ts">
import { computed } from 'vue';
import type { Device } from '@/types/ticket';

const props = defineProps<Device>();

const emit = defineEmits<{
  (e: 'remove'): void;
  (e: 'view', deviceId: number): void;
}>();

// Function to handle view button click
const handleViewClick = () => {
  emit('view', props.id);
};

// Computed property for warranty status styling
const warrantyStatusClass = computed(() => {
  switch (props.warranty_status) {
    case 'Active':
      return 'bg-green-900/30 text-green-400 border-green-700/30';
    case 'Warning':
      return 'bg-yellow-900/30 text-yellow-400 border-yellow-700/30';
    case 'Expired':
      return 'bg-red-900/30 text-red-400 border-red-700/30';
    default:
      return 'bg-gray-900/30 text-gray-400 border-gray-700/30';
  }
});
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 overflow-hidden hover:border-slate-600/50 transition-colors">
    <!-- Header with device name and actions -->
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3 min-w-0 flex-1">
          <div class="w-2 h-2 bg-blue-500 rounded-full flex-shrink-0"></div>
          <h3 class="font-medium text-white truncate">{{ hostname || 'Unknown Device' }}</h3>
          <div v-if="warranty_status" 
               class="px-2 py-1 rounded-md text-xs font-medium border flex-shrink-0"
               :class="warrantyStatusClass">
            {{ warranty_status }}
          </div>
        </div>
        
        <!-- Action buttons -->
        <div class="flex items-center gap-1 ml-2">
          <button
            @click="handleViewClick"
            class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded-md transition-colors"
            title="View device details"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
          </button>
          <button
            @click="emit('remove')"
            class="p-1.5 text-slate-400 hover:text-red-400 hover:bg-red-900/20 rounded-md transition-colors"
            title="Remove device"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Compact device information -->
    <div class="p-4">
      <div class="flex flex-col gap-3">
        <!-- Primary info row -->
        <div class="grid grid-cols-2 gap-3 text-sm">
          <div class="flex flex-col gap-1">
            <span class="text-xs text-slate-400 uppercase tracking-wide">Serial</span>
            <span class="text-slate-200 font-mono text-xs">{{ serial_number || 'N/A' }}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-xs text-slate-400 uppercase tracking-wide">Model</span>
            <span class="text-slate-200 text-xs truncate">{{ model || 'Unknown' }}</span>
          </div>
        </div>
        
        <!-- Quick action button -->
        <div class="pt-2 border-t border-slate-700/50">
          <button
            @click="handleViewClick"
            class="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-600/10 text-blue-400 rounded-lg hover:bg-blue-600/20 transition-colors text-sm font-medium"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            View Details
          </button>
        </div>
      </div>
    </div>
  </div>
</template>