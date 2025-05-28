<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ref, onMounted, computed, watch } from 'vue'
import BaseListView from '@/components/common/BaseListView.vue'
import DebouncedSearchInput from '@/components/common/DebouncedSearchInput.vue'
import PaginationControls from '@/components/common/PaginationControls.vue'
import Modal from '@/components/Modal.vue'
import DeviceForm from '@/components/DeviceForm.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import { getPaginatedDevices, createDevice } from '@/services/deviceService'
import type { Device, DeviceFormData } from '@/types/device'
import type { PaginatedResponse } from '@/services/deviceService'

const router = useRouter()
const devices = ref<Device[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const selectedDevices = ref<number[]>([])
const lastSelectedDeviceId = ref<number | null>(null)

// Add state for the modal
const showAddDeviceModal = ref(false)
const isCreatingDevice = ref(false)
const createDeviceError = ref<string | null>(null)

// Sorting state
const sortField = ref<string>("name")
const sortDirection = ref<"asc" | "desc">("asc")

// Search and filter state
const searchQuery = ref("")
const manufacturerFilter = ref<string>("all")
const warrantyFilter = ref<string>("all")

// Pagination state
const currentPage = ref(1)
const pageSize = ref(25)
const pageSizeOptions = [10, 25, 50, 100]
const totalItems = ref(0)
const totalPages = ref(1)

// Define columns for the table
const columns = [
  { field: 'id', label: 'ID', width: 'w-16 flex-shrink-0' },
  { field: 'name', label: 'Device Name', width: 'flex-1 min-w-0' },
  { field: 'manufacturer', label: 'Manufacturer', width: 'flex-1 min-w-0' },
  { field: 'model', label: 'Model', width: 'flex-1 min-w-0' },
  { field: 'primary_user', label: 'Primary User', width: 'flex-1 min-w-0' },
  { field: 'warranty_status', label: 'Warranty', width: 'w-24 flex-shrink-0' },
  { field: 'updated_at', label: 'Last Updated', width: 'flex-1 min-w-0' }
];

// Function to format date in locale format
const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    // Check if date is valid
    if (isNaN(date.getTime())) {
      return 'Unknown'
    }

    // Format date in user's locale
    return date.toLocaleString()
  } catch (error) {
    console.error("Error formatting date:", error)
    return 'Unknown'
  }
}

// Fetch devices from API with pagination
const fetchDevices = async () => {
  // Don't show loading for subsequent searches to prevent input focus loss
  if (devices.value.length === 0) {
    loading.value = true
  }
  error.value = null

  try {
    // For server-side pagination
    const response = await getPaginatedDevices({
      page: currentPage.value,
      pageSize: pageSize.value,
      sortField: sortField.value,
      sortDirection: sortDirection.value,
      search: searchQuery.value,
      type: manufacturerFilter.value !== 'all' ? manufacturerFilter.value : undefined,
      warranty: warrantyFilter.value !== 'all' ? warrantyFilter.value : undefined
    });
    
    devices.value = response.data;
    totalItems.value = response.total;
    totalPages.value = response.totalPages;
  } catch (err) {
    console.error("Failed to fetch devices:", err)
    error.value = "Failed to load devices. Please try again later."
  } finally {
    loading.value = false
  }
}

// Load devices when component mounts
onMounted(() => {
  fetchDevices()
})

// Watch for changes in pagination, sorting, or filtering to refetch data
watch(
  [currentPage, pageSize, sortField, sortDirection, searchQuery, manufacturerFilter, warrantyFilter],
  () => {
    fetchDevices();
  }
);

// Get unique manufacturers from devices (for filter options)
const availableManufacturers = computed(() => {
  // Common manufacturers from Microsoft Intune data
  return ['Microsoft Corporation', 'Dell Inc.', 'HP Inc.', 'Lenovo', 'Apple Inc.', 'ASUS', 'Acer'];
})

// Get unique warranty statuses from devices (for filter options)
const availableWarrantyStatuses = computed(() => {
  // Based on our compliance state mapping
  return ['Active', 'Warning', 'Expired', 'Unknown'];
})

// Prepare filter options for BaseListView
const filterOptions = computed(() => {
  return [
    {
      name: 'type',
      value: manufacturerFilter.value,
      options: [
        { value: 'all', label: 'All Manufacturers' },
        ...availableManufacturers.value.map(manufacturer => ({ value: manufacturer, label: manufacturer }))
      ],
      width: 'w-[160px]'
    },
    {
      name: 'warranty',
      value: warrantyFilter.value,
      options: [
        { value: 'all', label: 'All Warranty' },
        ...availableWarrantyStatuses.value.map(status => ({ value: status, label: status }))
      ],
      width: 'w-[120px]'
    }
  ]
})

// Reset all filters
const resetFilters = () => {
  searchQuery.value = ""
  manufacturerFilter.value = "all"
  warrantyFilter.value = "all"
  currentPage.value = 1
  fetchDevices()
}

// Handle filter updates from BaseListView
const handleFilterUpdate = (name: string, value: string) => {
  if (name === 'type') {
    manufacturerFilter.value = value
  } else if (name === 'warranty') {
    warrantyFilter.value = value
  }
  currentPage.value = 1 // Reset to first page when filters change
}

// Handle sort update from BaseListView
const handleSortUpdate = (field: string, direction: 'asc' | 'desc') => {
  sortField.value = field;
  sortDirection.value = direction;
  currentPage.value = 1 // Reset to first page when sort changes
};

const toggleSelection = (event: Event, deviceId: number) => {
  event.stopPropagation()

  // Handle shift key for range selection
  if (
    event instanceof MouseEvent &&
    event.shiftKey &&
    lastSelectedDeviceId.value !== null
  ) {
    const currentIndex = devices.value.findIndex(
      (device) => device.id === deviceId
    )
    const lastIndex = devices.value.findIndex(
      (device) => device.id === lastSelectedDeviceId.value
    )

    if (currentIndex !== -1 && lastIndex !== -1) {
      const startIndex = Math.min(currentIndex, lastIndex)
      const endIndex = Math.max(currentIndex, lastIndex)

      const devicesToSelect = devices.value
        .slice(startIndex, endIndex + 1)
        .map((device) => device.id)

      // Add all devices in range to selection if they're not already selected
      devicesToSelect.forEach((id) => {
        if (!selectedDevices.value.includes(id)) {
          selectedDevices.value.push(id)
        }
      })
    }
  } 
  // Handle Ctrl/Cmd key for toggling individual items without affecting others
  else if (event instanceof MouseEvent && (event.ctrlKey || event.metaKey)) {
    const index = selectedDevices.value.indexOf(deviceId)
    if (index === -1) {
      selectedDevices.value.push(deviceId)
    } else {
      selectedDevices.value.splice(index, 1)
    }
    
    // Update last selected device
    lastSelectedDeviceId.value = deviceId
  }
  // Regular single selection toggle (clears other selections if not using modifier keys)
  else {
    const index = selectedDevices.value.indexOf(deviceId);
    if (index === -1) {
      // Add to selection without clearing others
      selectedDevices.value.push(deviceId);
    } else {
      // Remove from selection
      selectedDevices.value.splice(index, 1);
    }

    // Update last selected device
    lastSelectedDeviceId.value = deviceId;
  }
}

const toggleAllDevices = (event: Event) => {
  event.stopPropagation()
  const checkbox = event.target as HTMLInputElement
  
  // If we're checking the box, select all visible devices
  if (checkbox.checked) {
    selectedDevices.value = devices.value.map((device) => device.id)
  } 
  // If unchecking, clear all selections
  else {
    selectedDevices.value = []
  }
  
  // Reset last selected device
  lastSelectedDeviceId.value = null
}

const navigateToDevice = (deviceId: number) => {
  router.push(`/devices/${deviceId}`)
}

const handleAddDevice = () => {
  showAddDeviceModal.value = true
}

// Function to handle device creation
const handleCreateDevice = async (deviceData: DeviceFormData) => {
  isCreatingDevice.value = true
  createDeviceError.value = null
  
  try {
    await createDevice(deviceData)
    showAddDeviceModal.value = false
    // Refresh the device list
    await fetchDevices()
  } catch (err) {
    console.error('Error creating device:', err)
    createDeviceError.value = 'Failed to create device. Please try again.'
  } finally {
    isCreatingDevice.value = false
  }
}

// Handle page change
const handlePageChange = (page: number) => {
  currentPage.value = page;
};

// Handle page size change
const handlePageSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1; // Reset to first page when changing page size
};
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Search and filter bar - OUTSIDE of BaseListView -->
    <div class="sticky top-0 z-20 bg-slate-800 border-b border-slate-700 shadow-md">
      <div class="p-2 flex items-center gap-2 flex-wrap">
        <!-- Search input - completely isolated -->
        <DebouncedSearchInput
          v-model="searchQuery"
          :placeholder="`Search devices...`"
        />

        <!-- Filters -->
        <template v-if="filterOptions.length > 0">
          <div 
            v-for="filter in filterOptions" 
            :key="filter.name"
            :class="[filter.width || 'w-[120px]']"
          >
            <select
              :value="filter.value"
              @change="e => handleFilterUpdate(filter.name, (e.target as HTMLSelectElement).value)"
              class="bg-slate-700 border border-slate-600 text-white text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 block w-full py-1 px-2"
            >
              <option
                v-for="option in filter.options"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Reset filters button -->
          <button
            @click="resetFilters"
            class="px-2 py-1 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:ring-2 focus:outline-none focus:ring-blue-800"
          >
            Reset
          </button>
        </template>

        <!-- Add button -->
        <button
          @click="handleAddDevice"
          class="px-2 py-1 text-xs font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:ring-2 focus:outline-none focus:ring-green-800 ml-auto"
        >
          Add Device
        </button>

        <!-- Results count -->
        <div class="text-xs text-gray-400">
          {{ totalItems }} result{{ totalItems !== 1 ? "s" : "" }}
        </div>
      </div>
    </div>

    <!-- List View - WITHOUT search - flex-1 to take remaining space -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <BaseListView
      title="Devices"
      :search-query="''" 
      :is-loading="loading"
      :is-empty="devices.length === 0 && !loading"
      :error="error"
      :filters="[]"
      :results-count="totalItems"
      :selected-items="selectedDevices.map(id => id.toString())"
      :visible-items="devices"
      :item-id-field="'id'"
      :enable-selection="true"
      :sort-field="sortField"
      :sort-direction="sortDirection"
      :columns="columns"
      :show-add-button="false"
      @update:filter="handleFilterUpdate"
      @update:sort="handleSortUpdate"
      @toggle-selection="(event, id) => toggleSelection(event, parseInt(id, 10))"
      @toggle-all="toggleAllDevices"
      @retry="fetchDevices"
    >
      <!-- Desktop Table View -->
      <template #default>
        <div class="min-w-[700px]">
          <div
            v-for="device in devices"
            :key="device.id"
            class="flex items-center h-12 border-b border-slate-800 text-sm text-gray-200 hover:bg-slate-800/50 transition-colors cursor-pointer"
            @click="navigateToDevice(device.id)"
          >
            <div class="flex items-center justify-center px-3 w-16 flex-shrink-0">
              <input
                type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedDevices.includes(device.id)"
                @click.stop="(event) => toggleSelection(event, device.id)"
              />
            </div>
            <div class="flex items-center px-3 w-16 flex-shrink-0 text-xs text-gray-400">
              #{{ device.id }}
            </div>
            <div class="flex items-center px-3 flex-1 min-w-0">
              <div class="truncate font-medium">{{ device.name }}</div>
            </div>
            <div class="flex items-center px-3 flex-1 min-w-0">
              <div class="truncate text-sm">{{ device.manufacturer || 'Unknown' }}</div>
            </div>
            <div class="flex items-center px-3 flex-1 min-w-0">
              <div class="truncate text-sm">{{ device.model }}</div>
            </div>
            <div class="flex items-center px-3 flex-1 min-w-0">
              <UserAvatar v-if="device.primary_user" :name="device.primary_user.uuid" size="sm" />
              <span v-else class="text-sm text-gray-500">Unassigned</span>
            </div>
            <div class="flex items-center justify-center px-3 w-24 flex-shrink-0">
              <span 
                class="text-xs px-2 py-1 rounded-full whitespace-nowrap"
                :class="{
                  'text-green-400 bg-green-900/20': device.warranty_status === 'Active',
                  'text-yellow-400 bg-yellow-900/20': device.warranty_status === 'Warning',
                  'text-red-400 bg-red-900/20': device.warranty_status === 'Expired',
                  'text-gray-400 bg-gray-900/20': device.warranty_status === 'Unknown'
                }"
              >
                {{ device.warranty_status }}
              </span>
            </div>
            <div class="flex items-center px-3 flex-1 min-w-0">
              <div class="truncate text-sm text-gray-400">
                {{ device.updated_at ? formatDate(device.updated_at) : 'Never' }}
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- Mobile Card View -->
      <template #mobile-view>
        <div class="space-y-2 p-2">
          <div
            v-for="device in devices"
            :key="device.id"
            @click="navigateToDevice(device.id)"
            class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
          >
            <div class="flex items-center gap-3">
              <div class="flex-shrink-0">
                <input
                  type="checkbox"
                  class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                  :checked="selectedDevices.includes(device.id)"
                  @click.stop="(event) => toggleSelection(event, device.id)"
                />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between mb-2">
                  <div class="font-medium truncate text-white">{{ device.name }}</div>
                  <div class="text-xs text-gray-400 ml-2">#{{ device.id }}</div>
                </div>
                <div class="flex flex-wrap gap-1 mb-2">
                  <span class="bg-slate-700/50 px-2 py-1 rounded text-xs">
                    {{ device.manufacturer || 'Unknown' }}
                  </span>
                  <span class="bg-slate-700/50 px-2 py-1 rounded text-xs">
                    {{ device.model }}
                  </span>
                  <span 
                    class="px-2 py-1 rounded text-xs"
                    :class="{
                      'bg-green-900/30 text-green-400': device.warranty_status === 'Active',
                      'bg-yellow-900/30 text-yellow-400': device.warranty_status === 'Warning',
                      'bg-red-900/30 text-red-400': device.warranty_status === 'Expired',
                      'bg-gray-900/30 text-gray-400': device.warranty_status === 'Unknown'
                    }"
                  >
                    {{ device.warranty_status }}
                  </span>
                </div>
                <div class="flex items-center justify-between text-xs">
                  <div v-if="device.primary_user" class="flex items-center gap-2">
                    <span class="text-gray-400">Assigned to:</span>
                    <UserAvatar :name="device.primary_user.uuid" size="xs" />
                  </div>
                  <div v-else class="text-gray-500">Unassigned</div>
                  <div class="text-gray-400">
                    {{ device.updated_at ? formatDate(device.updated_at) : 'Never' }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
              </template>
      </BaseListView>
    </div>

    <!-- Pagination Controls -->
    <PaginationControls
      :current-page="currentPage"
      :total-pages="totalPages"
      :page-size="pageSize"
      :page-size-options="pageSizeOptions"
      :show-import="true"
      @update:current-page="handlePageChange"
      @update:page-size="handlePageSizeChange"
      @import="() => {}"
    />

    <!-- Add Device Modal -->
    <Modal
      :show="showAddDeviceModal"
      title="Add New Device"
      @close="showAddDeviceModal = false"
    >
      <div v-if="createDeviceError" class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-sm text-white">
        {{ createDeviceError }}
      </div>
      
      <DeviceForm
        @submit="handleCreateDevice"
        @cancel="showAddDeviceModal = false"
      />
      
      <div v-if="isCreatingDevice" class="mt-4 flex justify-center">
        <div class="animate-spin rounded-full h-6 w-6 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
/* Optional: Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}
</style> 