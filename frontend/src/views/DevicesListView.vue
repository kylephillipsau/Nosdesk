<script setup lang="ts">
import { useRouter } from 'vue-router'
import { ref, onMounted, computed } from 'vue'
import BaseListView from '@/components/common/BaseListView.vue'
import Modal from '@/components/Modal.vue'
import DeviceForm from '@/components/DeviceForm.vue'
import { getDevices, createDevice } from '@/services/deviceService'
import type { Device, DeviceFormData } from '@/types/device'

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
const typeFilter = ref<string>("all")
const warrantyFilter = ref<string>("all")

// Define columns for the table
const columns = [
  { field: 'id', label: 'ID', width: 'w-20 flex-shrink-0' },
  { field: 'name', label: 'Name', width: 'flex-1 min-w-0' },
  { field: 'type', label: 'Type', width: 'w-32 flex-shrink-0' },
  { field: 'model', label: 'Model', width: 'w-32 flex-shrink-0' },
  { field: 'warranty_status', label: 'Warranty', width: 'w-32 flex-shrink-0' },
  { field: 'lastSeen', label: 'Last Seen', width: 'w-40 flex-shrink-0' }
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

// Fetch devices from API
const fetchDevices = async () => {
  loading.value = true
  error.value = null

  try {
    devices.value = await getDevices()
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

// Filter devices based on search query and filters
const filteredDevices = computed(() => {
  if (!devices.value.length) return []

  return devices.value.filter((device) => {
    // Text search (case insensitive)
    const searchLower = searchQuery.value.toLowerCase()
    const matchesSearch =
      searchQuery.value === "" ||
      device.name.toLowerCase().includes(searchLower) ||
      device.hostname.toLowerCase().includes(searchLower) ||
      device.serial_number.toLowerCase().includes(searchLower) ||
      device.model.toLowerCase().includes(searchLower) ||
      (device.type && device.type.toLowerCase().includes(searchLower))

    // Type filter
    const matchesType =
      typeFilter.value === "all" || 
      (device.type && device.type === typeFilter.value)

    // Warranty filter
    const matchesWarranty =
      warrantyFilter.value === "all" ||
      device.warranty_status === warrantyFilter.value

    return matchesSearch && matchesType && matchesWarranty
  })
})

// Sort devices based on current sort field and direction
const sortedDevices = computed(() => {
  if (!filteredDevices.value.length) return []

  return [...filteredDevices.value].sort((a, b) => {
    let valueA, valueB

    // Extract the values to compare based on the sort field
    switch (sortField.value) {
      case "id":
        valueA = a.id
        valueB = b.id
        break
      case "name":
        valueA = a.name.toLowerCase()
        valueB = b.name.toLowerCase()
        break
      case "type":
        valueA = (a.type || '').toLowerCase()
        valueB = (b.type || '').toLowerCase()
        break
      case "model":
        valueA = a.model.toLowerCase()
        valueB = b.model.toLowerCase()
        break
      case "warranty_status":
        valueA = a.warranty_status.toLowerCase()
        valueB = b.warranty_status.toLowerCase()
        break
      case "lastSeen":
        valueA = a.lastSeen ? new Date(a.lastSeen).getTime() : 0
        valueB = b.lastSeen ? new Date(b.lastSeen).getTime() : 0
        break
      default:
        valueA = a.id
        valueB = b.id
    }

    // Compare the values based on sort direction
    if (sortDirection.value === "asc") {
      return valueA > valueB ? 1 : valueA < valueB ? -1 : 0
    } else {
      return valueA < valueB ? 1 : valueA > valueB ? -1 : 0
    }
  })
})

// Get unique device types from devices
const availableTypes = computed(() => {
  if (!devices.value.length) return []
  const types = new Set(devices.value.map((device) => device.type).filter(Boolean))
  return Array.from(types) as string[]
})

// Get unique warranty statuses from devices
const availableWarrantyStatuses = computed(() => {
  if (!devices.value.length) return []
  const statuses = new Set(devices.value.map((device) => device.warranty_status))
  return Array.from(statuses) as string[]
})

// Prepare filter options for BaseListView
const filterOptions = computed(() => {
  return [
    {
      name: 'type',
      value: typeFilter.value,
      options: [
        { value: 'all', label: 'All Types' },
        ...availableTypes.value.map(type => ({ value: type, label: type }))
      ],
      width: 'w-[120px]'
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
  typeFilter.value = "all"
  warrantyFilter.value = "all"
}

// Handle filter updates from BaseListView
const handleFilterUpdate = (name: string, value: string) => {
  if (name === 'type') {
    typeFilter.value = value
  } else if (name === 'warranty') {
    warrantyFilter.value = value
  }
}

// Handle sort update from BaseListView
const handleSortUpdate = (field: string, direction: 'asc' | 'desc') => {
  sortField.value = field;
  sortDirection.value = direction;
};

const toggleSelection = (event: Event, deviceId: number) => {
  event.stopPropagation()

  // Handle shift key for range selection
  if (
    event instanceof MouseEvent &&
    event.shiftKey &&
    lastSelectedDeviceId.value !== null
  ) {
    const currentIndex = sortedDevices.value.findIndex(
      (device) => device.id === deviceId
    )
    const lastIndex = sortedDevices.value.findIndex(
      (device) => device.id === lastSelectedDeviceId.value
    )

    if (currentIndex !== -1 && lastIndex !== -1) {
      const startIndex = Math.min(currentIndex, lastIndex)
      const endIndex = Math.max(currentIndex, lastIndex)

      const devicesToSelect = sortedDevices.value
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
    selectedDevices.value = sortedDevices.value.map((device) => device.id)
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
</script>

<template>
  <div>
    <BaseListView
      title="Devices"
      :search-query="searchQuery"
      :is-loading="loading"
      :is-empty="sortedDevices.length === 0"
      :error="error"
      :filters="filterOptions"
      :results-count="sortedDevices.length"
      :selected-items="selectedDevices.map(id => id.toString())"
      :visible-items="sortedDevices"
      :item-id-field="'id'"
      :enable-selection="true"
      :sort-field="sortField"
      :sort-direction="sortDirection"
      :columns="columns"
      @update:search-query="value => searchQuery = value"
      @update:filter="handleFilterUpdate"
      @update:sort="handleSortUpdate"
      @toggle-selection="(event, id) => toggleSelection(event, parseInt(id, 10))"
      @toggle-all="toggleAllDevices"
      @reset-filters="resetFilters"
      @add="handleAddDevice"
      @retry="fetchDevices"
    >
      <!-- Desktop Table View -->
      <template #default>
        <div class="min-w-[960px]">
          <div
            v-for="device in sortedDevices"
            :key="device.id"
            class="flex border-b border-slate-800 text-sm text-gray-200 hover:bg-slate-800/50 transition-colors cursor-pointer gap-1"
            @click="navigateToDevice(device.id)"
          >
            <div class="flex items-center p-3 w-10 flex-shrink-0">
              <input
                type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedDevices.includes(device.id)"
                @click.stop="(event) => toggleSelection(event, device.id)"
              />
            </div>
            <div class="flex items-center p-3 w-20 flex-shrink-0">
              #{{ device.id }}
            </div>
            <div class="flex items-center p-3 flex-1 min-w-0">
              <div class="truncate">{{ device.name }}</div>
            </div>
            <div class="flex items-center p-3 w-32 flex-shrink-0">
              {{ device.type || 'N/A' }}
            </div>
            <div class="flex items-center p-3 w-32 flex-shrink-0">
              {{ device.model }}
            </div>
            <div class="flex items-center p-3 w-32 flex-shrink-0">
              <span 
                :class="{
                  'text-green-400': device.warranty_status === 'active',
                  'text-yellow-400': device.warranty_status === 'expiring',
                  'text-red-400': device.warranty_status === 'expired'
                }"
              >
                {{ device.warranty_status }}
              </span>
            </div>
            <div class="flex items-center p-3 w-40 flex-shrink-0">
              {{ device.lastSeen ? formatDate(device.lastSeen) : 'Never' }}
            </div>
          </div>
        </div>
      </template>

      <!-- Mobile Card View -->
      <template #mobile-view>
        <div class="space-y-2 p-2">
          <div
            v-for="device in sortedDevices"
            :key="device.id"
            @click="navigateToDevice(device.id)"
            class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
          >
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0">
                <input
                  type="checkbox"
                  class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                  :checked="selectedDevices.includes(device.id)"
                  @click.stop="(event) => toggleSelection(event, device.id)"
                />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <div class="font-medium truncate">{{ device.name }}</div>
                  <div class="text-xs text-gray-400 ml-2">#{{ device.id }}</div>
                </div>
                <div class="mt-2 flex flex-wrap gap-2 text-xs">
                  <div class="bg-slate-700/50 px-2 py-1 rounded">
                    {{ device.type || 'N/A' }}
                  </div>
                  <div class="bg-slate-700/50 px-2 py-1 rounded">
                    {{ device.model }}
                  </div>
                  <div 
                    class="px-2 py-1 rounded"
                    :class="{
                      'bg-green-900/30 text-green-400': device.warranty_status === 'active',
                      'bg-yellow-900/30 text-yellow-400': device.warranty_status === 'expiring',
                      'bg-red-900/30 text-red-400': device.warranty_status === 'expired'
                    }"
                  >
                    {{ device.warranty_status }}
                  </div>
                </div>
                <div class="mt-2 text-xs text-gray-400">
                  Last seen: {{ device.lastSeen ? formatDate(device.lastSeen) : 'Never' }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </BaseListView>

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