<!-- components/ticketComponents/DeviceSelectionModal.vue -->
<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import Modal from '@/components/Modal.vue';
import UserAvatar from '@/components/UserAvatar.vue';
import * as deviceService from '@/services/deviceService';
import type { Device } from '@/types/device';

const router = useRouter();
const props = defineProps<{
  show: boolean;
  currentTicketId?: number;
  existingDeviceIds?: number[];
  requesterUuid?: string; // Add requester UUID for prioritization
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select-device', device: Device): void;
}>();

const searchQuery = ref('');
const devices = ref<Device[]>([]);
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
  } catch (err) {
    console.error('Error loading devices:', err);
    error.value = 'Failed to load devices. Please try again.';
  } finally {
    loading.value = false;
  }
};

// Computed property for filtered and sorted devices
const filteredDevices = computed(() => {
  let filtered = devices.value;
  
  // Apply search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    filtered = filtered.filter((device: Device) => 
      device.name.toLowerCase().includes(query) || 
      device.hostname.toLowerCase().includes(query) ||
      device.serial_number.toLowerCase().includes(query) ||
      device.model.toLowerCase().includes(query) ||
      device.manufacturer?.toLowerCase().includes(query) ||
      device.primary_user?.name.toLowerCase().includes(query) ||
      device.primary_user?.email.toLowerCase().includes(query) ||
      String(device.id).includes(query)
    );
  }
  
  // Sort devices with prioritization
  return filtered.sort((a, b) => {
    // Priority 1: Intune-managed devices where primary user matches requester
    const aIsRequesterIntune = a.intune_device_id && a.primary_user_uuid === props.requesterUuid;
    const bIsRequesterIntune = b.intune_device_id && b.primary_user_uuid === props.requesterUuid;
    
    if (aIsRequesterIntune && !bIsRequesterIntune) return -1;
    if (!aIsRequesterIntune && bIsRequesterIntune) return 1;
    
    // Priority 2: Any Intune-managed devices
    const aIsIntune = !!a.intune_device_id;
    const bIsIntune = !!b.intune_device_id;
    
    if (aIsIntune && !bIsIntune) return -1;
    if (!aIsIntune && bIsIntune) return 1;
    
    // Priority 3: Devices with assigned users
    const aHasUser = !!a.primary_user;
    const bHasUser = !!b.primary_user;
    
    if (aHasUser && !bHasUser) return -1;
    if (!aHasUser && bHasUser) return 1;
    
    // Final sort: alphabetical by name
    return a.name.localeCompare(b.name);
  });
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

// Get device type based on manufacturer and model
const getDeviceType = (device: Device): string => {
  const manufacturer = device.manufacturer?.toLowerCase() || '';
  const model = device.model?.toLowerCase() || '';
  
  if (manufacturer.includes('microsoft') && model.includes('surface')) {
    return 'Surface';
  } else if (model.includes('laptop') || model.includes('thinkpad') || model.includes('latitude')) {
    return 'Laptop';
  } else if (model.includes('desktop') || model.includes('optiplex') || model.includes('thinkcentre')) {
    return 'Desktop';
  } else if (model.includes('tablet') || model.includes('ipad')) {
    return 'Tablet';
  } else if (model.includes('phone') || model.includes('iphone')) {
    return 'Mobile';
  }
  return 'Computer';
};

const getDeviceTypeClass = (device: Device) => {
  const type = getDeviceType(device);
  switch (type) {
    case 'Surface':
      return 'bg-blue-900/30 text-blue-300 border-blue-700/30';
    case 'Laptop':
      return 'bg-purple-900/30 text-purple-300 border-purple-700/30';
    case 'Desktop':
      return 'bg-green-900/30 text-green-300 border-green-700/30';
    case 'Tablet':
      return 'bg-yellow-900/30 text-yellow-300 border-yellow-700/30';
    case 'Mobile':
      return 'bg-pink-900/30 text-pink-300 border-pink-700/30';
    default:
      return 'bg-slate-700/30 text-slate-300 border-slate-600/30';
  }
};

// Get warranty status styling
const getWarrantyStatusClass = (status: string) => {
  switch (status.toLowerCase()) {
    case 'active':
    case 'compliant':
      return 'bg-green-900/30 text-green-400 border-green-700/30';
    case 'warning':
    case 'noncompliant':
      return 'bg-yellow-900/30 text-yellow-400 border-yellow-700/30';
    case 'expired':
    case 'error':
      return 'bg-red-900/30 text-red-400 border-red-700/30';
    default:
      return 'bg-slate-700/30 text-slate-400 border-slate-600/30';
  }
};

// Check if device is prioritized
const isPrioritizedDevice = (device: Device): boolean => {
  return !!(device.intune_device_id && device.primary_user_uuid === props.requesterUuid);
};

// Format last updated date
const formatLastUpdated = (dateString: string): string => {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffTime / (1000 * 60));

    if (diffMinutes < 1) {
      return 'just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes}m ago`;
    } else if (diffMinutes < 1440) {
      const hours = Math.floor(diffMinutes / 60);
      return `${hours}h ago`;
    } else {
      const days = Math.floor(diffMinutes / 1440);
      return `${days}d ago`;
    }
  } catch (e) {
    return 'unknown';
  }
};
</script>

<template>
  <Modal :show="show" title="Select a Device" @close="emit('close')" size="lg">
    <div class="flex flex-col gap-4">
      <!-- Search -->
      <div class="relative">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <svg class="h-5 w-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <input 
          type="text" 
          v-model="searchQuery"
          class="w-full pl-10 pr-4 py-3 rounded-lg border border-slate-600 bg-slate-700 text-white placeholder-slate-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-colors"
          placeholder="Search devices by name, hostname, serial number, manufacturer, or user..."
        >
      </div>

      <!-- Loading state -->
      <div v-if="loading" class="text-center py-8 text-slate-400">
        <div class="inline-flex items-center gap-3">
          <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Loading devices...</span>
        </div>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="text-center py-8">
        <div class="bg-red-900/20 border border-red-700/30 rounded-lg p-4">
          <p class="text-red-400 flex items-center justify-center gap-2">
            <svg class="w-5 h-5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            {{ error }}
          </p>
        </div>
      </div>

      <!-- Devices list -->
      <div v-else class="max-h-[500px] overflow-y-auto">
        <div v-if="filteredDevices.length === 0" class="text-center py-8 text-slate-400">
          <div class="inline-flex flex-col items-center gap-3">
            <svg class="w-12 h-12 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            <span>No devices found</span>
          </div>
        </div>
        
        <!-- Compact table-like layout -->
        <div v-else class="bg-slate-800 rounded-lg border border-slate-700/50 overflow-hidden">
          <!-- Table header -->
          <div class="bg-slate-700/50 px-4 py-3 border-b border-slate-600/50">
            <div class="grid grid-cols-12 gap-3 text-xs font-medium text-slate-300 uppercase tracking-wide">
              <div class="col-span-3">Device</div>
              <div class="col-span-2">Type & Status</div>
              <div class="col-span-2">Serial Number</div>
              <div class="col-span-3">Primary User</div>
              <div class="col-span-1">Updated</div>
              <div class="col-span-1 text-right">Action</div>
            </div>
          </div>
          
          <!-- Device rows -->
          <div class="divide-y divide-slate-700/30">
            <div 
              v-for="device in filteredDevices" 
              :key="device.id"
              class="group relative hover:bg-slate-700/30 transition-colors duration-150 cursor-pointer"
              :class="{ 'bg-blue-900/20 border-l-4 border-blue-500': isPrioritizedDevice(device) }"
              @click="selectDevice(device)"
            >
              <!-- Priority indicator -->
              <div v-if="isPrioritizedDevice(device)" class="absolute -top-1 right-2 z-10">
                <div class="bg-blue-500 text-white text-xs px-2 py-0.5 rounded-b-md shadow-sm">
                  Requester's Device
                </div>
              </div>

              <div class="px-4 py-3">
                <div class="grid grid-cols-12 gap-3 items-center">
                  <!-- Device Name & Info -->
                  <div class="col-span-3 min-w-0">
                    <div class="flex flex-col gap-1">
                      <div class="font-medium text-white truncate text-sm">{{ device.name }}</div>
                      <div class="text-xs text-slate-400 truncate">{{ device.hostname }}</div>
                      <div class="text-xs text-slate-500 truncate">{{ device.manufacturer || 'Unknown' }} {{ device.model }}</div>
                    </div>
                  </div>

                  <!-- Type & Status -->
                  <div class="col-span-2 min-w-0">
                    <div class="flex flex-wrap gap-1">
                      <span 
                        class="text-xs px-2 py-1 rounded-full border"
                        :class="getDeviceTypeClass(device)"
                      >
                        {{ getDeviceType(device) }}
                      </span>
                      <span 
                        class="text-xs px-2 py-1 rounded-full border"
                        :class="getWarrantyStatusClass(device.warranty_status)"
                      >
                        {{ device.warranty_status }}
                      </span>
                      <span v-if="device.intune_device_id" class="text-xs px-2 py-1 rounded-full bg-blue-900/30 text-blue-300 border border-blue-700/30">
                        Intune
                      </span>
                    </div>
                  </div>

                  <!-- Serial Number -->
                  <div class="col-span-2 min-w-0">
                    <span class="text-sm text-slate-300 font-mono truncate block">{{ device.serial_number }}</span>
                  </div>

                  <!-- Primary User -->
                  <div class="col-span-3 min-w-0">
                    <div v-if="device.primary_user" class="flex items-center gap-2">
                      <UserAvatar :name="device.primary_user.uuid" size="sm" />
                      <div class="flex-1 min-w-0">
                        <div class="text-sm font-medium text-slate-200 truncate">{{ device.primary_user.name }}</div>
                        <div class="text-xs text-slate-400 truncate">{{ device.primary_user.email }}</div>
                      </div>
                    </div>
                    <div v-else class="flex items-center gap-2 text-slate-500">
                      <div class="w-6 h-6 rounded-full bg-slate-700 flex items-center justify-center">
                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd" />
                        </svg>
                      </div>
                      <span class="text-xs">Unassigned</span>
                    </div>
                  </div>

                  <!-- Last Updated -->
                  <div class="col-span-1 min-w-0">
                    <span class="text-xs text-slate-400">{{ formatLastUpdated(device.updated_at) }}</span>
                  </div>

                  <!-- Action Button -->
                  <div class="col-span-1 text-right">
                    <button class="text-blue-400 hover:text-blue-300 text-xs font-medium px-2 py-1 rounded hover:bg-blue-900/20 transition-colors">
                      Select
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="mt-6 flex justify-between items-center pt-4 border-t border-slate-700">
      <button 
        type="button"
        class="flex items-center gap-2 px-4 py-2 text-sm text-blue-400 hover:text-blue-300 hover:bg-blue-900/20 rounded-md transition-colors"
        @click="$router.push('/devices/new')"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        </svg>
        Create New Device
      </button>
      
      <div class="flex items-center gap-3">
        <span class="text-sm text-slate-400">{{ filteredDevices.length }} device{{ filteredDevices.length !== 1 ? 's' : '' }} available</span>
        <button 
          type="button"
          class="px-4 py-2 text-sm text-slate-300 hover:text-slate-100 hover:bg-slate-700 rounded-md transition-colors"
          @click="emit('close')"
        >
          Cancel
        </button>
      </div>
    </div>
  </Modal>
</template> 