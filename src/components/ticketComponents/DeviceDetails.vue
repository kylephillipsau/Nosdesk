<!-- components/DeviceDetails.vue -->
<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import type { Device } from '@/types/ticket';

const props = defineProps<Device>();

type DeviceFieldKey = 'hostname' | 'serialNumber' | 'model' | 'warrantyStatus';

const editableDevice = reactive({
  hostname: props.hostname,
  serialNumber: props.serialNumber,
  model: props.model,
  warrantyStatus: props.warrantyStatus
});

const isEditing = ref(false); // Start in view mode

const emit = defineEmits<{
  (e: 'remove'): void;
  (e: 'view'): void;
}>();

// Define device fields with their display labels
const deviceFields = [
  { key: 'hostname' as DeviceFieldKey, label: 'Hostname' },
  { key: 'serialNumber' as DeviceFieldKey, label: 'Serial Number' },
  { key: 'model' as DeviceFieldKey, label: 'Model' },
  { key: 'warrantyStatus' as DeviceFieldKey, label: 'Warranty Status' }
];

// Function to start editing
const startEditing = () => {
  isEditing.value = true;
};

// Function to stop editing when clicking outside
const stopEditing = () => {
  isEditing.value = false;
};

// Handle clicks outside of the editable inputs
const handleClickOutside = (event: MouseEvent) => {
  if (!(event.target instanceof HTMLElement) || !event.target.closest('.input-wrapper')) {
    stopEditing();
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div class="bg-slate-800 rounded-lg overflow-hidden">
    <div class="px-4 py-3 bg-slate-700/50 flex items-center justify-between">
      <h2 class="text-lg font-medium text-slate-100">Device</h2>
      <div class="flex items-center gap-2">
        <button
          @click="emit('view')"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
          title="View device details"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
            <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
          </svg>
        </button>
        <button
          @click="emit('remove')"
          class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
          title="Remove device"
        >
          <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
    </div>
    <div class="p-2">
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <!-- Use v-for to iterate through device fields -->
        <div 
          v-for="field in deviceFields" 
          :key="field.key"
          class="flex flex-col gap-1 bg-slate-700 p-2 rounded-xl shadow-inner input-wrapper"
        >
          <dt class="text-sm text-slate-400">{{ field.label }}</dt>
          <input 
            v-model="editableDevice[field.key]" 
            :class="{'focus:bg-slate-600': isEditing}" 
            @click="startEditing" 
            class="text-slate-200 bg-slate-600 border-none p-2 rounded transition-colors duration-150" 
          />
        </div>
      </div>
    </div>
  </div>
</template>