<script setup lang="ts">
import { useRouter } from 'vue-router'
import BaseListView from '@/components/common/BaseListView.vue'
import SelectableTable from '@/components/common/SelectableTable.vue'
import { useDataLoader } from '@/composables/useDataLoader'
import { useSearch } from '@/composables/useSearch'
import { useSelection } from '@/composables/useSelection'

interface Device {
  id: string
  name: string
  type: string
  model: string
  serialNumber: string
  status: 'active' | 'inactive'
  lastSeen: string
  assignedTo: string | null
}

const router = useRouter()

// Data loading
const { data: devices, isLoading } = useDataLoader<Device>({
  fetchData: async () => {
    // TODO: Replace with actual API call
    return [
      {
        id: '1',
        name: 'MacBook Pro 16"',
        type: 'Laptop',
        model: 'MacBook Pro M2',
        serialNumber: 'MBP2023001',
        status: 'active',
        lastSeen: '2024-03-15T10:30:00Z',
        assignedTo: 'john.doe'
      },
      {
        id: '2',
        name: 'iPhone 15 Pro',
        type: 'Mobile',
        model: 'iPhone 15 Pro Max',
        serialNumber: 'IP15P2023002',
        status: 'active',
        lastSeen: '2024-03-15T09:45:00Z',
        assignedTo: 'jane.smith'
      },
      {
        id: '3',
        name: 'Dell XPS 15',
        type: 'Laptop',
        model: 'XPS 15 9570',
        serialNumber: 'DXP2023003',
        status: 'inactive',
        lastSeen: '2024-02-28T14:20:00Z',
        assignedTo: null
      },
      {
        id: '4',
        name: 'iPad Pro 12.9"',
        type: 'Tablet',
        model: 'iPad Pro M2',
        serialNumber: 'IPP2023004',
        status: 'active',
        lastSeen: '2024-03-14T16:45:00Z',
        assignedTo: 'jane.smith'
      }
    ]
  }
})

// Search functionality
const { searchQuery, filteredItems: filteredDevices } = useSearch<Device>({
  items: devices,
  searchableFields: [
    device => device.name,
    device => device.type,
    device => device.model,
    device => device.serialNumber,
    device => device.assignedTo || ''
  ]
})

// Selection functionality
const { selectedIds, toggleSelection, toggleAll } = useSelection<Device>({
  items: devices,
  getItemId: device => device.id
})

// Table columns
const columns = [
  { key: 'device', label: 'Device' },
  { key: 'type', label: 'Type' },
  { key: 'serialNumber', label: 'Serial Number', hidden: true },
  { key: 'model', label: 'Model' },
  { key: 'lastSeen', label: 'Last Seen', hidden: true },
  { key: 'assignedTo', label: 'Assigned To', hidden: true }
]

const navigateToDevice = (device: Device) => {
  router.push(`/devices/${device.id}`)
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString()
}

const handleAddDevice = () => {
  // TODO: Implement add device functionality
  console.log('Add device clicked')
}
</script>

<template>
  <BaseListView
    title="Devices"
    :search-query="searchQuery"
    :is-loading="isLoading"
    :is-empty="filteredDevices.length === 0"
    @update:search-query="value => searchQuery = value"
    @add="handleAddDevice"
  >
    <SelectableTable
      :items="filteredDevices"
      :columns="columns"
      :selected-ids="selectedIds"
      @toggle-selection="toggleSelection"
      @toggle-all="toggleAll"
      @row-click="navigateToDevice"
    >
      <!-- Custom cell rendering -->
      <template #cell="{ item: device, column }">
        <!-- Device column with icon and name -->
        <td v-if="column.key === 'device'" class="px-6 py-4 whitespace-nowrap">
          <div class="flex items-center gap-2">
            <div class="flex-shrink-0 h-10 w-10 bg-slate-700 rounded-full flex items-center justify-center">
              <svg class="h-6 w-6 text-slate-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path v-if="device.type === 'Laptop'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                <path v-else-if="device.type === 'Mobile'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18v-5.25m0 0a6.01 6.01 0 001.5-.189m-1.5.189a6.01 6.01 0 01-1.5-.189m3.75 7.478a12.06 12.06 0 01-4.5 0m3.75 2.383a14.406 14.406 0 01-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 10-7.517 0c.85.493 1.509 1.333 1.509 2.316V18" />
                <path v-else-if="device.type === 'Tablet'" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <div class="ml-4">
              <div class="text-sm font-medium">{{ device.name }}</div>
              <div class="text-sm text-gray-400 md:hidden">{{ device.serialNumber }}</div>
              <div class="text-sm text-gray-400 md:hidden">{{ device.assignedTo || 'Unassigned' }}</div>
            </div>
          </div>
        </td>
        
        <!-- Last Seen column with formatted date -->
        <td v-else-if="column.key === 'lastSeen'" :class="['px-6 py-4 whitespace-nowrap text-sm', column.hidden ? 'hidden md:table-cell' : '']">
          {{ formatDate(device.lastSeen) }}
        </td>
        
        <!-- Assigned To column with unassigned fallback -->
        <td v-else-if="column.key === 'assignedTo'" :class="['px-6 py-4 whitespace-nowrap text-sm', column.hidden ? 'hidden md:table-cell' : '']">
          {{ device.assignedTo || 'Unassigned' }}
        </td>
        
        <!-- Default rendering for other columns -->
        <td v-else :class="['px-6 py-4 whitespace-nowrap text-sm', column.hidden ? 'hidden md:table-cell' : '']">
          {{ device[column.key as keyof Device] }}
        </td>
      </template>
      
      <template #empty>
        No devices found matching your search.
      </template>
    </SelectableTable>
  </BaseListView>
</template> 