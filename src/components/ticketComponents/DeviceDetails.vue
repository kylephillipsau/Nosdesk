<!-- components/DeviceDetails.vue -->
<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import type { Device } from '@/types/ticket';

const props = defineProps<Device>();

const editableDevice = reactive({
  hostname: props.hostname,
  serialNumber: props.serialNumber,
  model: props.model,
  warrantyStatus: props.warrantyStatus
});

const isEditing = ref(true); // Start in edit mode

const emit = defineEmits<{
  (e: 'remove'): void;
}>();

// Function to start editing (in this case, it's always true)
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
      <h2 class="text-lg font-medium text-slate-100">{{ name }}</h2>
      <button 
        @click="emit('remove')" 
        class="text-red-400 hover:text-red-300 text-sm transition-colors"
      >
        Remove
      </button>
    </div>
    <div class="p-4">
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <!-- Hostname -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner input-wrapper">
          <dt class="text-sm text-slate-400">Hostname</dt>
          <input 
            v-model="editableDevice.hostname" 
            :class="{'focus:bg-slate-600': isEditing}" 
            @click="startEditing" 
            class="text-slate-200 bg-slate-600 border-none p-1 rounded transition-colors duration-150" 
          />
        </div>

        <!-- Serial Number -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner input-wrapper">
          <dt class="text-sm text-slate-400">Serial Number</dt>
          <input 
            v-model="editableDevice.serialNumber" 
            :class="{'focus:bg-slate-600': isEditing}" 
            @click="startEditing" 
            class="text-slate-200 bg-slate-600 border-none p-1 rounded transition-colors duration-150" 
          />
        </div>

        <!-- Model -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner input-wrapper">
          <dt class="text-sm text-slate-400">Model</dt>
          <input 
            v-model="editableDevice.model" 
            :class="{'focus:bg-slate-600': isEditing}" 
            @click="startEditing" 
            class="text-slate-200 bg-slate-600 border-none p-1 rounded transition-colors duration-150" 
          />
        </div>

        <!-- Warranty Status -->
        <div class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner input-wrapper">
          <dt class="text-sm text-slate-400">Warranty Status</dt>
          <input 
            v-model="editableDevice.warrantyStatus" 
            :class="{'focus:bg-slate-600': isEditing}" 
            @click="startEditing" 
            class="text-slate-200 bg-slate-600 border-none p-1 rounded transition-colors duration-150" 
          />
        </div>
      </div>
    </div>
  </div>
</template>