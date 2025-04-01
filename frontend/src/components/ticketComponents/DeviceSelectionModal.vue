<!-- components/ticketComponents/DeviceSelectionModal.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import Modal from '@/components/Modal.vue';
import * as deviceService from '@/services/deviceService';
import type { Device } from '@/types/device';

const router = useRouter();
const props = defineProps<{
  show: boolean;
  currentTicketId?: number;
  existingDeviceIds?: number[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select-device', device: Device): void;
}>();

const searchQuery = ref('');
const devices = ref<Device[]>([]);
const filteredDevices = ref<Device[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

// Load devices from the API
const loadDevices = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const allDevices = await deviceService.getDevices();
    console.log(`Total devices fetched: ${allDevices.length}`);
    
    // Filter out any already assigned devices
    if (props.existingDeviceIds && props.existingDeviceIds.length > 0) {
      devices.value = allDevices.filter((device: Device) => 
        !props.existingDeviceIds?.includes(device.id)
      );
      console.log(`Filtered out ${allDevices.length - devices.value.length} already assigned devices`);
    } else {
      devices.value = allDevices;
    }
    
    console.log(`Displaying ${devices.value.length} available devices for selection`);
    filterDevices();
  } catch (err) {
    console.error('Error loading devices:', err);
    error.value = 'Failed to load devices. Please try again.';
  } finally {
    loading.value = false;
  }
};

// Filter devices based on search query
const filterDevices = () => {
  if (!searchQuery.value.trim()) {
    filteredDevices.value = devices.value;
    return;
  }
  
  const query = searchQuery.value.toLowerCase();
  filteredDevices.value = devices.value.filter((device: Device) => 
    device.name.toLowerCase().includes(query) || 
    device.hostname.toLowerCase().includes(query) ||
    device.serial_number.toLowerCase().includes(query) ||
    device.model.toLowerCase().includes(query) ||
    String(device.id).includes(query)
  );
};

// Watch for search query changes
watch(searchQuery, () => {
  filterDevices();
});

// Watch for modal visibility
watch(() => props.show, (newValue) => {
  if (newValue) {
    loadDevices();
    searchQuery.value = '';
  }
});

const selectDevice = (device: Device) => {
  emit('select-device', device);
  emit('close');
};

const getDeviceTypeClass = (type: string = '') => {
  switch (type.toLowerCase()) {
    case 'laptop':
      return 'bg-blue-900 text-blue-300';
    case 'desktop':
      return 'bg-purple-900 text-purple-300';
    case 'mobile':
      return 'bg-green-900 text-green-300';
    case 'tablet':
      return 'bg-yellow-900 text-yellow-300';
    default:
      return 'bg-gray-800 text-gray-300';
  }
};
</script>

<template>
  <Modal :show="show" title="Select a Device" @close="emit('close')">
    <div class="flex flex-col gap-2">
      <!-- Search -->
      <div>
        <input type="text" 
          v-model="searchQuery"
          class="w-full p-2 rounded-lg border-gray-600 bg-slate-700 text-white placeholder-gray-400 focus:border-blue-500 focus:ring-blue-500"
          placeholder="Search devices by name, hostname, serial number..."
        >
      </div>

      <!-- Loading state -->
      <div v-if="loading" class="text-center py-4 text-gray-400">
        Loading devices...
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="text-center py-4 text-red-400">
        {{ error }}
      </div>

      <!-- Devices list -->
      <div v-else class="max-h-96 overflow-y-auto">
        <div v-if="filteredDevices.length === 0" class="text-center py-4 text-gray-400">
          No devices found
        </div>
        <div v-else class="space-y-1">
          <div v-for="device in filteredDevices" :key="device.id"
            class="group px-3 py-2 rounded-lg transition-colors duration-200 hover:bg-slate-700 relative cursor-pointer"
            @click="selectDevice(device)">
            <!-- Device info -->
            <div class="flex items-center justify-between">
              <div class="flex flex-col gap-1 flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-white">{{ device.name }}</span>
                  <span 
                    class="text-xs px-2 py-0.5 rounded-full"
                    :class="getDeviceTypeClass(device.type)"
                  >
                    {{ device.type }}
                  </span>
                </div>
                <div class="flex items-center gap-4 text-xs text-gray-400">
                  <span>{{ device.hostname }}</span>
                  <span>{{ device.model }}</span>
                </div>
                <div class="text-xs text-gray-500">
                  S/N: {{ device.serial_number }}
                </div>
              </div>
              
              <button 
                class="text-blue-400 hover:text-blue-300 text-sm">
                Select
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-6 flex justify-between">
      <button type="button"
        class="px-4 py-2 text-sm text-blue-500 hover:text-blue-400"
        @click="$router.push('/devices/new')">
        + Create New Device
      </button>
      <button type="button"
        class="px-4 py-2 text-sm text-slate-300 hover:text-slate-100"
        @click="emit('close')">
        Cancel
      </button>
    </div>
  </Modal>
</template> 