<script setup lang="ts">
import { ref, defineEmits, defineProps } from 'vue';
import type { DeviceFormData } from '@/types/device';

const props = defineProps<{
  initialData?: Partial<DeviceFormData>;
}>();

const emit = defineEmits<{
  (e: 'submit', data: DeviceFormData): void;
  (e: 'cancel'): void;
}>();

// Form data with default values
const formData = ref<DeviceFormData>({
  name: props.initialData?.name || '',
  hostname: props.initialData?.hostname || '',
  serial_number: props.initialData?.serial_number || '',
  model: props.initialData?.model || '',
  warranty_status: props.initialData?.warranty_status || 'Unknown',
  type: props.initialData?.type || 'Laptop',
});

// Form validation
const errors = ref<Record<string, string>>({});

const validateForm = (): boolean => {
  const newErrors: Record<string, string> = {};
  
  if (!formData.value.name.trim()) {
    newErrors.name = 'Name is required';
  }
  
  if (!formData.value.hostname.trim()) {
    newErrors.hostname = 'Hostname is required';
  }
  
  if (!formData.value.serial_number.trim()) {
    newErrors.serial_number = 'Serial number is required';
  }
  
  if (!formData.value.model.trim()) {
    newErrors.model = 'Model is required';
  }
  
  errors.value = newErrors;
  return Object.keys(newErrors).length === 0;
};

const handleSubmit = () => {
  if (validateForm()) {
    emit('submit', { ...formData.value });
  }
};

const handleCancel = () => {
  emit('cancel');
};

// Device type options
const deviceTypes = [
  { value: 'Laptop', label: 'Laptop' },
  { value: 'Desktop', label: 'Desktop' },
  { value: 'Mobile', label: 'Mobile' },
  { value: 'Tablet', label: 'Tablet' },
  { value: 'Other', label: 'Other' }
];

// Warranty status options
const warrantyStatuses = [
  { value: 'Active', label: 'Active' },
  { value: 'Expired', label: 'Expired' },
  { value: 'Unknown', label: 'Unknown' }
];
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-4">
    <!-- Device Name -->
    <div>
      <label for="name" class="block text-sm font-medium text-gray-300">Device Name</label>
      <input
        id="name"
        v-model="formData.name"
        type="text"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
        :class="{ 'border-red-500': errors.name }"
      />
      <p v-if="errors.name" class="mt-1 text-sm text-red-500">{{ errors.name }}</p>
    </div>
    
    <!-- Hostname -->
    <div>
      <label for="hostname" class="block text-sm font-medium text-gray-300">Hostname</label>
      <input
        id="hostname"
        v-model="formData.hostname"
        type="text"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
        :class="{ 'border-red-500': errors.hostname }"
      />
      <p v-if="errors.hostname" class="mt-1 text-sm text-red-500">{{ errors.hostname }}</p>
    </div>
    
    <!-- Serial Number -->
    <div>
      <label for="serial_number" class="block text-sm font-medium text-gray-300">Serial Number</label>
      <input
        id="serial_number"
        v-model="formData.serial_number"
        type="text"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
        :class="{ 'border-red-500': errors.serial_number }"
      />
      <p v-if="errors.serial_number" class="mt-1 text-sm text-red-500">{{ errors.serial_number }}</p>
    </div>
    
    <!-- Model -->
    <div>
      <label for="model" class="block text-sm font-medium text-gray-300">Model</label>
      <input
        id="model"
        v-model="formData.model"
        type="text"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
        :class="{ 'border-red-500': errors.model }"
      />
      <p v-if="errors.model" class="mt-1 text-sm text-red-500">{{ errors.model }}</p>
    </div>
    
    <!-- Device Type -->
    <div>
      <label for="type" class="block text-sm font-medium text-gray-300">Device Type</label>
      <select
        id="type"
        v-model="formData.type"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
      >
        <option v-for="option in deviceTypes" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>
    
    <!-- Warranty Status -->
    <div>
      <label for="warranty_status" class="block text-sm font-medium text-gray-300">Warranty Status</label>
      <select
        id="warranty_status"
        v-model="formData.warranty_status"
        class="mt-1 block w-full bg-gray-700 border border-gray-600 rounded-md shadow-sm py-2 px-3 text-white focus:outline-none focus:ring-blue-500 focus:border-blue-500"
      >
        <option v-for="option in warrantyStatuses" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>
    
    <!-- Form Actions -->
    <div class="flex justify-end space-x-3 pt-4">
      <button
        type="button"
        @click="handleCancel"
        class="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-500 focus:outline-none focus:ring-2 focus:ring-gray-500"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        Save
      </button>
    </div>
  </form>
</template> 