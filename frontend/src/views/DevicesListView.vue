<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import BaseListView from '@/components/common/BaseListView.vue'
import DataTable from '@/components/common/DataTable.vue'
import DebouncedSearchInput from '@/components/common/DebouncedSearchInput.vue'
import PaginationControls from '@/components/common/PaginationControls.vue'
import Modal from '@/components/Modal.vue'
import DeviceForm from '@/components/DeviceForm.vue'
import { IdCell, TextCell, StatusBadgeCell, UserAvatarCell } from '@/components/common/cells'
import UserAvatar from '@/components/UserAvatar.vue'
import { useListManagement } from '@/composables/useListManagement'
import { getPaginatedDevices, createDevice } from '@/services/deviceService'
import type { Device, DeviceFormData } from '@/types/device'

const router = useRouter()

// Use the composable for all common functionality
const listManager = useListManagement<Device>({
  itemIdField: 'id',
  defaultSortField: 'name',
  defaultSortDirection: 'asc',
  fetchFunction: async (params) => {
    return await getPaginatedDevices({
      page: params.page,
      pageSize: params.pageSize,
      sortField: params.sortField,
      sortDirection: params.sortDirection,
      search: params.search,
      type: params.type,
      warranty: params.warranty
    });
  },
  routeBuilder: (device) => `/devices/${device.id}`
});

// Define table columns with responsive behavior
const columns = [
  { field: 'id', label: 'ID', width: 'minmax(60px,auto)', sortable: true, responsive: 'md' as const },
  { field: 'name', label: 'Device Name', width: '1fr', sortable: true, responsive: 'always' as const },
  { field: 'serial_number', label: 'Serial Number', width: 'minmax(120px,auto)', sortable: true, responsive: 'md' as const },
  { field: 'manufacturer', label: 'Manufacturer', width: 'minmax(120px,auto)', sortable: true, responsive: 'lg' as const },
  { field: 'model', label: 'Model', width: 'minmax(120px,auto)', sortable: true, responsive: 'lg' as const },
  { field: 'primary_user', label: 'Primary User', width: 'minmax(120px,auto)', sortable: false, responsive: 'md' as const },
  { field: 'warranty_status', label: 'Warranty', width: 'minmax(100px,auto)', sortable: true, responsive: 'always' as const }
];

// Get available filter options
const availableManufacturers = computed(() => {
  return ['Microsoft Corporation', 'Dell Inc.', 'HP Inc.', 'Lenovo', 'Apple Inc.', 'ASUS', 'Acer'];
});

const availableWarrantyStatuses = computed(() => {
  return ['Active', 'Warning', 'Expired', 'Unknown'];
});

// Build filter options
const filterOptions = computed(() => {
  return listManager.buildFilterOptions({
    type: {
      options: availableManufacturers.value.map(manufacturer => ({ 
        value: manufacturer.toLowerCase(), 
        label: manufacturer 
      })),
      width: 'w-[160px]',
      allLabel: 'All Manufacturers'
    },
    warranty: {
      options: availableWarrantyStatuses.value.map(status => ({ 
        value: status.toLowerCase(), 
        label: status 
      })),
      width: 'w-[120px]',
      allLabel: 'All Warranties'
    }
  });
});

// Custom grid template for responsive layout
const gridClass = "grid-cols-[auto_1fr_minmax(100px,auto)] md:grid-cols-[auto_minmax(60px,auto)_1fr_minmax(120px,auto)_minmax(120px,auto)_minmax(100px,auto)] lg:grid-cols-[auto_minmax(60px,auto)_1fr_minmax(120px,auto)_minmax(120px,auto)_minmax(120px,auto)_minmax(120px,auto)_minmax(100px,auto)]";

// Modal state
const showAddDeviceModal = ref(false);
const isCreatingDevice = ref(false);
const createDeviceError = ref<string | null>(null);

// Handle device creation
const handleCreateDevice = async (deviceData: DeviceFormData) => {
  isCreatingDevice.value = true;
  createDeviceError.value = null;
  
  try {
    await createDevice(deviceData);
    showAddDeviceModal.value = false;
    await listManager.fetchItems();
  } catch (err) {
    console.error('Error creating device:', err);
    createDeviceError.value = 'Failed to create device. Please try again.';
  } finally {
    isCreatingDevice.value = false;
  }
};

// Format date function
const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString);
    if (isNaN(date.getTime())) return 'Unknown';
    return date.toLocaleString();
  } catch (error) {
    console.error("Error formatting date:", error);
    return 'Unknown';
  }
};
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Search and filter bar -->
    <div class="sticky top-0 z-20 bg-slate-800 border-b border-slate-700 shadow-md">
      <div class="p-2 flex items-center gap-2 flex-wrap">
        <DebouncedSearchInput
          v-model="listManager.searchQuery.value"
          placeholder="Search devices..."
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
              @change="e => listManager.handleFilterUpdate(filter.name, (e.target as HTMLSelectElement).value)"
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

          <button
            @click="listManager.resetFilters"
            class="px-2 py-1 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:ring-2 focus:outline-none focus:ring-blue-800"
          >
            Reset
          </button>
        </template>

        <!-- Add button -->
        <button
          @click="showAddDeviceModal = true"
          class="px-2 py-1 text-xs font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:ring-2 focus:outline-none focus:ring-green-800 ml-auto"
        >
          Add Device
        </button>

        <div class="text-xs text-slate-400">
          {{ listManager.totalItems.value }} result{{ listManager.totalItems.value !== 1 ? "s" : "" }}
        </div>
      </div>
    </div>

    <!-- Main content -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <BaseListView
        title=""
        :search-query="''"
        :is-loading="listManager.loading.value"
        :is-empty="listManager.items.value.length === 0 && !listManager.loading.value"
        :error="listManager.error.value"
        :filters="[]"
        :results-count="listManager.totalItems.value"
        :selected-items="listManager.selectedItems.value"
        :visible-items="listManager.items.value"
        :item-id-field="'id'"
        :enable-selection="false"
        :sort-field="listManager.sortField.value"
        :sort-direction="listManager.sortDirection.value"
        :columns="[]"
        :show-add-button="false"
        @retry="listManager.fetchItems"
      >
        <!-- Desktop Table View -->
        <template #default>
          <DataTable
            :columns="columns"
            :data="listManager.items.value"
            :selected-items="listManager.selectedItems.value"
            :sort-field="listManager.sortField.value"
            :sort-direction="listManager.sortDirection.value"
            :grid-class="gridClass"
            @update:sort="listManager.handleSortUpdate"
            @toggle-selection="listManager.toggleSelection"
            @toggle-all="listManager.toggleAllItems"
            @row-click="listManager.navigateToItem"
          >
            <!-- Custom cell templates -->
            <template #cell-id="{ value }">
              <IdCell :id="value" />
            </template>
            
            <template #cell-name="{ value }">
              <TextCell :value="value" font-weight="medium" />
            </template>
            
            <template #cell-serial_number="{ value }">
              <TextCell :value="value" />
            </template>
            
            <template #cell-manufacturer="{ value }">
              <TextCell :value="value || 'Unknown'" />
            </template>
            
            <template #cell-model="{ value }">
              <TextCell :value="value" />
            </template>
            
            <template #cell-primary_user="{ item }">
              <UserAvatarCell 
                v-if="item.primary_user"
                :user-id="item.primary_user.uuid" 
                :show-name="true" 
              />
              <span v-else class="text-xs text-slate-500">Unassigned</span>
            </template>
            
            <template #cell-warranty_status="{ value }">
              <StatusBadgeCell type="warranty" :value="value" />
            </template>
          </DataTable>
        </template>

        <!-- Mobile Card View -->
        <template #mobile-view>
          <div class="flex flex-col gap-2 p-2">
            <div
              v-for="device in listManager.items.value"
              :key="device.id"
              @click="listManager.navigateToItem(device)"
              class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
            >
              <div class="flex items-center gap-3">
                <div class="flex-grow-0">
                  <input
                    type="checkbox"
                    class="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-600 focus:ring-blue-500"
                    :checked="listManager.selectedItems.value.includes(device.id.toString())"
                    @click.stop="(event) => listManager.toggleSelection(event, device.id.toString())"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between mb-2">
                    <div class="font-medium truncate text-slate-200">{{ device.name }}</div>
                    <div class="text-xs text-slate-400 ml-2">#{{ device.id }}</div>
                  </div>
                  <div class="flex flex-wrap gap-1 mb-2">
                    <span class="bg-slate-700/50 px-2 py-1 rounded text-xs text-slate-300">
                      {{ device.manufacturer || 'Unknown' }}
                    </span>
                    <span class="bg-slate-700/50 px-2 py-1 rounded text-xs text-slate-300">
                      {{ device.model }}
                    </span>
                    <span class="bg-slate-700/50 px-2 py-1 rounded text-xs text-slate-300 font-mono">
                      SN: {{ device.serial_number }}
                    </span>
                    <span 
                      class="px-2 py-1 rounded text-xs"
                      :class="{
                        'bg-green-900/30 text-green-400': device.warranty_status === 'Active',
                        'bg-yellow-900/30 text-yellow-400': device.warranty_status === 'Warning',
                        'bg-red-900/30 text-red-400': device.warranty_status === 'Expired',
                        'bg-slate-700 text-slate-400': device.warranty_status === 'Unknown'
                      }"
                    >
                      {{ device.warranty_status }}
                    </span>
                  </div>
                  <div class="flex items-center justify-between text-xs">
                    <div v-if="device.primary_user" class="flex items-center gap-2">
                      <span class="text-slate-400">Assigned to:</span>
                      <UserAvatar :name="device.primary_user.uuid" size="xs" />
                    </div>
                    <div v-else class="text-slate-500">Unassigned</div>
                    <div class="text-slate-400">
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
      :current-page="listManager.currentPage.value"
      :total-pages="listManager.totalPages.value"
      :page-size="listManager.pageSize.value"
      :page-size-options="listManager.pageSizeOptions"
      :show-import="true"
      @update:current-page="listManager.handlePageChange"
      @update:page-size="listManager.handlePageSizeChange"
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
/* Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar,
.overflow-x-auto::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track,
.overflow-x-auto::-webkit-scrollbar-track {
  background: #0f172a; /* slate-900 */
}

.overflow-y-auto::-webkit-scrollbar-thumb,
.overflow-x-auto::-webkit-scrollbar-thumb {
  background: #475569; /* slate-600 */
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover,
.overflow-x-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b; /* slate-500 */
}

.overflow-x-auto::-webkit-scrollbar-corner {
  background: #0f172a; /* slate-900 */
}
</style> 