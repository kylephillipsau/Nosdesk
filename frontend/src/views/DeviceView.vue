<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import BackButton from '@/components/common/BackButton.vue';

interface Device {
  id: string;
  name: string;
  type: string;
  lastSeen: string;
  status: string;
  hostname: string;
  serialNumber: string;
  model: string;
  warrantyStatus: string;
  specs?: {
    cpu?: string;
    memory?: string;
    storage?: string;
    os?: string;
  };
}

const route = useRoute();
const device = ref<Device | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

const fetchDeviceData = async () => {
  try {
    loading.value = true;
    error.value = null;
    
    // TODO: Replace with actual API call
    // Mock data for now
    device.value = {
      id: route.params.id as string,
      name: 'MacBook Pro',
      type: 'Laptop',
      lastSeen: new Date().toISOString(),
      status: 'online',
      hostname: 'MBP-2021',
      serialNumber: 'FVFXC2YGDH8M',
      model: 'MacBook Pro (16-inch, 2021)',
      warrantyStatus: 'Active until Dec 2024',
      specs: {
        cpu: 'Apple M1 Pro',
        memory: '16GB',
        storage: '512GB SSD',
        os: 'macOS 14.0'
      }
    };
  } catch (e) {
    error.value = 'Failed to load device details';
    console.error('Error loading device:', e);
  } finally {
    loading.value = false;
  }
};

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffTime / (1000 * 60));

    if (diffMinutes < 1) {
      return 'just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes} minute${diffMinutes === 1 ? '' : 's'} ago`;
    } else if (diffMinutes < 1440) { // less than 24 hours
      const hours = Math.floor(diffMinutes / 60);
      return `${hours} hour${hours === 1 ? '' : 's'} ago`;
    } else {
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    }
  } catch (e) {
    return dateString;
  }
};

onMounted(() => {
  fetchDeviceData();
});
</script>

<template>
  <div class="flex-1">
    <div v-if="device" class="flex flex-col">
      <!-- Navigation and actions bar -->
      <div class="pt-4 px-6 flex justify-between items-center">
        <BackButton fallbackRoute="/" />
      </div>
      
      <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
        <!-- Device Header -->
        <div class="bg-slate-800 rounded-2xl p-6">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3">
                <h1 class="text-2xl font-semibold text-white">{{ device.name }}</h1>
              </div>
              <p class="text-slate-400 mt-1">{{ device.type }}</p>
              <p class="text-sm text-slate-500 mt-2">
                Last seen {{ formatDate(device.lastSeen) }}
              </p>
            </div>
          </div>
        </div>

        <!-- Device Details -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <!-- System Information -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <h2 class="text-lg font-medium text-white mb-4">System Information</h2>
            <div class="space-y-4">
              <div class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Hostname</span>
                <span class="text-white">{{ device.hostname }}</span>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Serial Number</span>
                <span class="text-white">{{ device.serialNumber }}</span>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Model</span>
                <span class="text-white">{{ device.model }}</span>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Warranty Status</span>
                <span class="text-white">{{ device.warrantyStatus }}</span>
              </div>
            </div>
          </div>

          <!-- Hardware Specifications -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <h2 class="text-lg font-medium text-white mb-4">Hardware Specifications</h2>
            <div class="space-y-4">
              <div v-if="device.specs?.cpu" class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Processor</span>
                <span class="text-white">{{ device.specs.cpu }}</span>
              </div>
              <div v-if="device.specs?.memory" class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Memory</span>
                <span class="text-white">{{ device.specs.memory }}</span>
              </div>
              <div v-if="device.specs?.storage" class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Storage</span>
                <span class="text-white">{{ device.specs.storage }}</span>
              </div>
              <div v-if="device.specs?.os" class="flex flex-col gap-1">
                <span class="text-sm text-slate-400">Operating System</span>
                <span class="text-white">{{ device.specs.os }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="loading" class="flex justify-center items-center min-h-[200px]">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
    </div>

    <div v-else class="p-6 text-center text-slate-400">
      Device not found
    </div>
  </div>
</template> 