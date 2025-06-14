<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import BaseListView from "@/components/common/BaseListView.vue";
import DataTable from "@/components/common/DataTable.vue";
import DebouncedSearchInput from "@/components/common/DebouncedSearchInput.vue";
import PaginationControls from "@/components/common/PaginationControls.vue";
import { IdCell, TextCell, StatusBadgeCell, UserInfoCell } from "@/components/common/cells";
import UserAvatar from "@/components/UserAvatar.vue";
import { useListManagement } from "@/composables/useListManagement";
import { useDataStore } from "@/stores/dataStore";
import { useOptimisticUpdates } from "@/composables/useOptimisticUpdates";
import type { User } from "@/services/userService";

// Extended user interface for UI display
interface UIUser extends User {
  department?: string;
}

const router = useRouter();
const dataStore = useDataStore();
const optimisticUpdates = useOptimisticUpdates();

// Use the composable for all common functionality
const listManager = useListManagement<UIUser>({
  itemIdField: 'uuid',
  defaultSortField: 'id',
  defaultSortDirection: 'asc',
  fetchFunction: async (params) => {
    // Check if we have cached data for this exact query
    const cacheKey = `users_${JSON.stringify(params)}`;
    
    console.log(`🔍 Fetching users with params:`, params);
    
    const response = await dataStore.getPaginatedUsers({
      page: params.page,
      pageSize: params.pageSize,
      sortField: params.sortField,
      sortDirection: params.sortDirection,
      search: params.search,
      role: params.role !== 'all' ? params.role : undefined
    });
    
    console.log(`📊 Received ${response.data.length} users (${response.total} total)`);
    
    // Transform backend users to UI users with additional properties
    const transformedData = response.data.map(user => ({
      ...user,
      department: "IT Support", // Default department (could be added to backend later)
    }));
    
    return {
      data: transformedData,
      total: response.total,
      totalPages: response.totalPages
    };
  },
  routeBuilder: (user) => `/users/${user.uuid}`
});

// Define table columns with responsive behavior
const columns = [
  { field: 'id', label: 'ID', width: 'minmax(60px,auto)', sortable: true, responsive: 'md' as const },
  { field: 'user', label: 'User', width: '1fr', sortable: false, responsive: 'always' as const },
  { field: 'role', label: 'Role', width: 'minmax(120px,auto)', sortable: true, responsive: 'md' as const },
  { field: 'department', label: 'Department', width: 'minmax(120px,auto)', sortable: false, responsive: 'lg' as const }
];

// Get available filter options
const availableRoles = computed(() => {
  return ['admin', 'user', 'technician'];
});

// Build filter options
const filterOptions = computed(() => {
  return listManager.buildFilterOptions({
    role: {
      options: availableRoles.value.map(role => ({ 
        value: role.toLowerCase(), 
        label: role.charAt(0).toUpperCase() + role.slice(1)
      })),
      width: 'w-[150px]',
      allLabel: 'All Roles'
    }
  });
});

// Custom grid template for responsive layout
const gridClass = "grid-cols-[auto_1fr_minmax(120px,auto)] md:grid-cols-[auto_minmax(60px,auto)_1fr_minmax(120px,auto)] lg:grid-cols-[auto_minmax(60px,auto)_1fr_minmax(120px,auto)_minmax(120px,auto)]";

// Navigate to user creation
const navigateToCreateUser = () => {
  router.push('/users/new');
};
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Search and filter bar -->
    <div class="sticky top-0 z-20 bg-slate-800 border-b border-slate-700 shadow-md">
      <div class="p-2 flex items-center gap-2 flex-wrap">
        <DebouncedSearchInput
          v-model="listManager.searchQuery.value"
          placeholder="Search users..."
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
          @click="navigateToCreateUser"
          class="px-2 py-1 text-xs font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:ring-2 focus:outline-none focus:ring-green-800 ml-auto"
        >
          Add User
        </button>

        <!-- Results count and cache stats -->
        <div class="text-xs text-slate-400 flex items-center gap-4">
          <span>{{ listManager.totalItems.value }} result{{ listManager.totalItems.value !== 1 ? "s" : "" }}</span>
          <span v-if="dataStore.getCacheStats.individualUsers > 0" class="text-blue-400">
            📦 {{ dataStore.getCacheStats.individualUsers }} cached
          </span>
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
        :sort-field="listManager.sortField.value"
        :sort-direction="listManager.sortDirection.value"
        :columns="[]"
        :selected-items="listManager.selectedItems.value"
        :visible-items="listManager.items.value"
        :item-id-field="'uuid'"
        :enable-selection="false"
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
            
            <template #cell-user="{ item }">
              <UserInfoCell 
                :user-id="item.uuid"
                :user-name="item.name"
                :user-email="item.email"
                :show-avatar="true"
                :show-name="true"
                :show-email="true"
              />
            </template>
            
            <template #cell-role="{ value }">
              <StatusBadgeCell type="role" :value="value" />
            </template>
            
            <template #cell-department="{ value }">
              <TextCell :value="value || 'N/A'" />
            </template>
          </DataTable>
        </template>

        <!-- Mobile Card View -->
        <template #mobile-view>
          <div class="flex flex-col gap-2 p-2">
            <div
              v-for="user in listManager.items.value"
              :key="user.uuid"
              @click="listManager.navigateToItem(user)"
              class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
            >
              <div class="flex items-start gap-3">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <UserAvatar
                      :name="user.uuid"
                      :userName="user.name"
                      size="sm"
                      :clickable="false"
                      :show-name="false"
                    />
                    <div class="flex-1 min-w-0">
                      <div class="font-medium truncate text-slate-200">{{ user.name }}</div>
                      <div class="text-xs text-slate-400 truncate">{{ user.email }}</div>
                    </div>
                    <div class="text-xs text-slate-400">#{{ user.id }}</div>
                  </div>
                  <div class="mt-2 flex flex-wrap gap-2 text-xs">
                    <span class="bg-slate-700 px-2 py-1 rounded text-slate-200">
                      {{ user.role }}
                    </span>
                    <span class="bg-slate-700/50 px-2 py-1 rounded text-slate-300">
                      {{ user.department || 'N/A' }}
                    </span>
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
