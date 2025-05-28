<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import BaseListView from "@/components/common/BaseListView.vue";
import DebouncedSearchInput from "@/components/common/DebouncedSearchInput.vue";
import PaginationControls from "@/components/common/PaginationControls.vue";
import UserAvatar from "@/components/UserAvatar.vue";
import StatusBadge from "@/components/StatusBadge.vue";
import Modal from "@/components/Modal.vue";
import UserForm from "@/components/UserForm.vue";
import userService from "@/services/userService";
import type { User, PaginatedResponse } from "@/services/userService";
import { useImagePreloader } from "@/utils/imagePreloader";
import { getListAvatarUrl, getProfileAvatarUrl } from "@/utils/avatarUtils";
import { useDataStore } from "@/stores/dataStore";
import { useOptimisticUpdates } from "@/composables/useOptimisticUpdates";

// Extended user interface for UI display
interface UIUser extends User {
  department?: string;
}

const router = useRouter();
const users = ref<UIUser[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const searchQuery = ref("");
const roleFilter = ref("all");
const imagesLoading = ref(false); // Track image loading state separately

// Add sorting state
const sortField = ref("id");
const sortDirection = ref<"asc" | "desc">("asc");

// Add selection state
const selectedUsers = ref<string[]>([]);
const lastSelectedUserUuid = ref<string | null>(null);

// Add state for the modal
const showAddUserModal = ref(false);
const isCreatingUser = ref(false);
const createUserError = ref<string | null>(null);

// Pagination state
const currentPage = ref(1);
const pageSize = ref(25);
const pageSizeOptions = [10, 25, 50, 100];
const totalItems = ref(0);
const totalPages = ref(1);

// Image preloader
const { preloadImages, clearQueue } = useImagePreloader();

// Global data store and optimistic updates
const dataStore = useDataStore();
const optimisticUpdates = useOptimisticUpdates();

// Track current page loading to cancel previous requests
const currentPageLoading = ref<AbortController | null>(null);

// Get unique roles from users (for filter options)
const availableRoles = computed(() => {
  // Since we're using server-side pagination, we'll provide static options
  // In a real application, you might want to fetch these from the server
  return ['admin', 'user', 'moderator', 'support'];
});

// Prepare filter options for BaseListView
const filterOptions = computed(() => {
  return [
    {
      name: 'role',
      value: roleFilter.value,
      options: [
        { value: 'all', label: 'All Roles' },
        ...availableRoles.value.map(role => ({ value: role, label: role }))
      ],
      width: 'w-[150px]'
    }
  ];
});

const navigateToUser = (uuid: string) => {
  router.push(`/users/${uuid}`);
};

// Preload avatar images for better performance with cancellation support
const preloadAvatars = async (userList: UIUser[], priority: 'high' | 'medium' = 'high') => {
  // Cancel any previous page loading
  if (currentPageLoading.value) {
    currentPageLoading.value.abort();
    clearQueue(); // Clear any pending requests
  }

  // Create new abort controller for this page
  const abortController = new AbortController();
  currentPageLoading.value = abortController;

  // Set global abort controller for image preloader
  const { setGlobalAbortController } = useImagePreloader();
  setGlobalAbortController(abortController);

  // Get both thumbnail and full avatar URLs
  const listAvatarUrls = userList
    .map(user => getListAvatarUrl(user.avatar_url || null, user.avatar_thumb || null))
    .filter(Boolean) as string[];
    
  const profileAvatarUrls = userList
    .map(user => getProfileAvatarUrl(user.avatar_url || null, user.avatar_thumb || null))
    .filter(Boolean) as string[];
  
  // Combine and deduplicate URLs
  const allAvatarUrls = [...new Set([...listAvatarUrls, ...profileAvatarUrls])];
  
  if (allAvatarUrls.length > 0) {
    imagesLoading.value = true; // Start image loading indicator
    try {
      // Preload list avatars with high priority (currently visible)
      if (listAvatarUrls.length > 0) {
        await preloadImages(listAvatarUrls, { 
          priority: 'high', 
          timeout: 3000,
          retries: 1,
          signal: abortController.signal
        });
      }
      
      // Only preload profile avatars if not cancelled
      if (!abortController.signal.aborted && profileAvatarUrls.length > 0) {
        await preloadImages(profileAvatarUrls, { 
          priority: 'medium', 
          timeout: 5000,
          retries: 1,
          signal: abortController.signal
        });
      }
    } catch (error) {
      if (!abortController.signal.aborted) {
        console.warn('Failed to preload some avatar images:', error);
      }
    } finally {
      if (!abortController.signal.aborted) {
        imagesLoading.value = false; // Stop image loading indicator
      }
    }
  }
};

// Fetch users from API with pagination using global data store
const fetchUsers = async () => {
  // Cancel any previous requests
  if (currentPageLoading.value) {
    currentPageLoading.value.abort();
  }
  
  loading.value = true;
  error.value = null;
  
  try {
    // Use the global data store for caching
    const response = await dataStore.getPaginatedUsers({
      page: currentPage.value,
      pageSize: pageSize.value,
      sortField: sortField.value,
      sortDirection: sortDirection.value,
      search: searchQuery.value,
      role: roleFilter.value !== 'all' ? roleFilter.value : undefined
    });
    
    // Transform backend users to UI users with additional properties
    users.value = response.data.map(user => ({
      ...user,
      department: "IT Support", // Default department (could be added to backend later),
    }));
    
    totalItems.value = response.total;
    totalPages.value = response.totalPages;
    
    // Preload avatars for the current page with high priority
    await preloadAvatars(users.value, 'high');
    
    // Prefetch next page avatars in the background (low priority) - but only if not cancelled
    if (!currentPageLoading.value?.signal.aborted) {
      prefetchNextPageAvatars();
    }
    
    console.log('Users loaded from cache/API:', users.value);
  } catch (err: any) {
    // Don't show error for cancelled requests
    if (err.message === 'REQUEST_CANCELLED') {
      console.log('User fetch request was cancelled');
      return;
    }
    
    console.error('Error fetching users:', err);
    error.value = 'Failed to load users. Please try again later.';
  } finally {
    loading.value = false;
  }
};

// Prefetch next page avatars in the background
const prefetchNextPageAvatars = async () => {
  // Only prefetch if there's a next page and we're not on the last page
  if (currentPage.value < totalPages.value && !currentPageLoading.value?.signal.aborted) {
    try {
      const nextPageResponse = await dataStore.getPaginatedUsers({
        page: currentPage.value + 1,
        pageSize: pageSize.value,
        sortField: sortField.value,
        sortDirection: sortDirection.value,
        search: searchQuery.value,
        role: roleFilter.value !== 'all' ? roleFilter.value : undefined
      });
      
      // Check if still not cancelled before processing
      if (!currentPageLoading.value?.signal.aborted) {
        const nextPageUsers = nextPageResponse.data;
        const nextPageAvatarUrls = nextPageUsers
          .map(user => getListAvatarUrl(user.avatar_url || null, user.avatar_thumb || null))
          .filter(Boolean) as string[];
        
        if (nextPageAvatarUrls.length > 0) {
          // Use low priority and idle time for prefetching
          await preloadImages(nextPageAvatarUrls, { 
            priority: 'low', 
            timeout: 10000,
            retries: 0, // No retries for prefetch
            signal: currentPageLoading.value?.signal
          });
        }
      }
    } catch (error: any) {
      // Silently fail for prefetch - it's not critical
      if (error.message !== 'REQUEST_CANCELLED') {
        console.debug('Next page prefetch failed:', error);
      }
    }
  }
};

// Load users when component mounts
onMounted(() => {
  fetchUsers();
});

// Cleanup on unmount
onUnmounted(() => {
  if (currentPageLoading.value) {
    currentPageLoading.value.abort();
  }
  clearQueue();
  userService.cancelAllRequests();
});

// Watch for changes in pagination, sorting, or filtering to refetch data
watch(
  [currentPage, pageSize, sortField, sortDirection, roleFilter],
  () => {
    fetchUsers();
  }
);

// Debounced search to prevent excessive API calls
let searchTimeout: number | null = null;
watch(searchQuery, (newValue) => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  
  searchTimeout = window.setTimeout(() => {
    currentPage.value = 1; // Reset to first page when searching
    fetchUsers();
  }, 300); // 300ms debounce
});

// Reset all filters
const resetFilters = () => {
  searchQuery.value = "";
  roleFilter.value = "all";
  currentPage.value = 1;
  fetchUsers();
};

// Handle filter updates from BaseListView
const handleFilterUpdate = (name: string, value: string) => {
  if (name === 'role') {
    roleFilter.value = value;
  }
  currentPage.value = 1; // Reset to first page when filters change
};

// Add function to handle user creation with optimistic updates
const handleCreateUser = async (userData: { name: string; email: string; role: string }) => {
  isCreatingUser.value = true;
  createUserError.value = null;
  
  try {
    await optimisticUpdates.createUser(userData);
    showAddUserModal.value = false;
    // Refresh the user list to show the new user
    await fetchUsers();
  } catch (err) {
    console.error('Error creating user:', err);
    createUserError.value = 'Failed to create user. Please try again.';
  } finally {
    isCreatingUser.value = false;
  }
};

// Selection functionality
const toggleSelection = (event: Event, userUuid: string) => {
  event.stopPropagation();

  // Handle shift key for range selection
  if (
    event instanceof MouseEvent &&
    event.shiftKey &&
    lastSelectedUserUuid.value !== null
  ) {
    const currentIndex = users.value.findIndex(
      (user) => user.uuid === userUuid
    );
    const lastIndex = users.value.findIndex(
      (user) => user.uuid === lastSelectedUserUuid.value
    );

    if (currentIndex !== -1 && lastIndex !== -1) {
      const startIndex = Math.min(currentIndex, lastIndex);
      const endIndex = Math.max(currentIndex, lastIndex);

      const usersToSelect = users.value
        .slice(startIndex, endIndex + 1)
        .map((user) => user.uuid);

      // Add all users in range to selection if they're not already selected
      usersToSelect.forEach((uuid) => {
        if (!selectedUsers.value.includes(uuid)) {
          selectedUsers.value.push(uuid);
        }
      });
    }
  } 
  // Handle Ctrl/Cmd key for toggling individual items without affecting others
  else if (event instanceof MouseEvent && (event.ctrlKey || event.metaKey)) {
    const index = selectedUsers.value.indexOf(userUuid);
    if (index === -1) {
      selectedUsers.value.push(userUuid);
    } else {
      selectedUsers.value.splice(index, 1);
    }
    
    // Update last selected user
    lastSelectedUserUuid.value = userUuid;
  }
  // Regular click - toggle the selection without clearing others
  else {
    const index = selectedUsers.value.indexOf(userUuid);
    if (index === -1) {
      // Add to selection without clearing others
      selectedUsers.value.push(userUuid);
    } else {
      // Remove from selection
      selectedUsers.value.splice(index, 1);
    }

    // Update last selected user
    lastSelectedUserUuid.value = userUuid;
  }
};

const toggleAllUsers = (event: Event) => {
  event.stopPropagation();
  const checkbox = event.target as HTMLInputElement;
  
  // If we're checking the box, select all visible users
  if (checkbox.checked) {
    selectedUsers.value = users.value.map((user) => user.uuid);
  } 
  // If unchecking, clear all selections
  else {
    selectedUsers.value = [];
  }
  
  // Reset last selected user
  lastSelectedUserUuid.value = null;
};

// Define columns for the table
const columns = [
  { field: 'id', label: 'ID', width: 'w-20 flex-shrink-0' },
  { field: 'name', label: 'User', width: 'flex-1 min-w-0' },
  { field: 'role', label: 'Role', width: 'w-32 flex-shrink-0' },
  { field: 'department', label: 'Department', width: 'w-32 flex-shrink-0' }
];

// Handle sort update from BaseListView
const handleSortUpdate = (field: string, direction: 'asc' | 'desc') => {
  sortField.value = field;
  sortDirection.value = direction;
  currentPage.value = 1; // Reset to first page when sort changes
};

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
          :placeholder="`Search users...`"
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
          @click="showAddUserModal = true"
          class="px-2 py-1 text-xs font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:ring-2 focus:outline-none focus:ring-green-800 ml-auto"
        >
          Add User
        </button>

        <!-- Results count and cache stats -->
        <div class="text-xs text-gray-400 flex items-center gap-4">
          <span>{{ totalItems }} result{{ totalItems !== 1 ? "s" : "" }}</span>
          <span v-if="dataStore.getCacheStats.individualUsers > 0" class="text-blue-400">
            📦 {{ dataStore.getCacheStats.individualUsers }} cached
          </span>
        </div>
      </div>
    </div>

    <!-- List View - WITHOUT search - flex-1 to take remaining space -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <BaseListView
      title="Users"
      :search-query="''"
      :is-loading="loading"
      :is-empty="users.length === 0 && !loading"
      :error="error"
      :filters="[]"
      :results-count="totalItems"
      :sort-field="sortField"
      :sort-direction="sortDirection"
      :columns="columns"
      :selected-items="selectedUsers"
      :visible-items="users"
      :item-id-field="'uuid'"
      :enable-selection="true"
      :show-add-button="false"
      @update:filter="handleFilterUpdate"
      @update:sort="handleSortUpdate"
      @toggle-selection="toggleSelection"
      @toggle-all="toggleAllUsers"
      @retry="fetchUsers"
    >
      <!-- Image loading indicator -->
      <template #header-extra>
        <div v-if="imagesLoading" class="flex items-center gap-2 text-sm text-gray-400">
          <div class="animate-spin rounded-full h-4 w-4 border-t-2 border-b-2 border-blue-500"></div>
          <span>Loading images...</span>
        </div>
      </template>

      <!-- Desktop Table View -->
      <template #default>
        <div class="min-w-[800px]">
          <div
            v-for="user in users"
            :key="user.uuid"
            @click="navigateToUser(user.uuid)"
            class="flex border-b border-slate-800 text-sm text-gray-200 hover:bg-slate-800/50 transition-colors cursor-pointer gap-1"
          >
            <div class="flex items-center p-3 w-10 flex-shrink-0">
              <input
                type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedUsers.includes(user.uuid)"
                @click.stop="(event) => toggleSelection(event, user.uuid)"
              />
            </div>
            <div class="flex items-center p-3 w-20 flex-shrink-0">
              #{{ user.id }}
            </div>
            <div class="p-3 flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <UserAvatar
                  :name="user.uuid"
                  :userName="user.name"
                  size="sm"
                  :clickable="false"
                  :show-name="false"
                />
                <div>
                  <div class="font-medium">{{ user.name }}</div>
                  <div class="text-xs text-gray-400">{{ user.email }}</div>
                </div>
              </div>
            </div>
            <div class="p-3 w-32 flex-shrink-0">
              {{ user.role }}
            </div>
            <div class="p-3 w-32 flex-shrink-0">
              {{ user.department || 'N/A' }}
            </div>
          </div>
        </div>
      </template>

      <!-- Mobile Card View -->
      <template #mobile-view>
        <div class="space-y-2 p-2">
          <div
            v-for="user in users"
            :key="user.uuid"
            @click="navigateToUser(user.uuid)"
            class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
          >
            <div class="flex items-start gap-3">
              <div class="flex-shrink-0">
                <input
                  type="checkbox"
                  class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                  :checked="selectedUsers.includes(user.uuid)"
                  @click.stop="(event) => toggleSelection(event, user.uuid)"
                />
              </div>
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
                    <div class="font-medium truncate">{{ user.name }}</div>
                    <div class="text-xs text-gray-400 truncate">{{ user.email }}</div>
                  </div>
                </div>
                <div class="mt-2 flex flex-wrap gap-2 text-xs">
                  <div class="bg-slate-700/50 px-2 py-1 rounded">
                    {{ user.role }}
                  </div>
                  <div class="bg-slate-700/50 px-2 py-1 rounded">
                    {{ user.department || 'N/A' }}
                  </div>
                  <div class="text-gray-400">
                    #{{ user.id }}
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

    <!-- Add User Modal -->
    <Modal
      :show="showAddUserModal"
      title="Add New User"
      @close="showAddUserModal = false"
    >
      <div v-if="createUserError" class="mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-sm text-white">
        {{ createUserError }}
      </div>
      
      <UserForm
        @submit="handleCreateUser"
        @cancel="showAddUserModal = false"
      />
      
      <div v-if="isCreatingUser" class="mt-4 flex justify-center">
        <div class="animate-spin rounded-full h-6 w-6 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
/* Custom scrollbar styling moved to BaseListView */
</style>
