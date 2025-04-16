<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import BaseListView from "@/components/common/BaseListView.vue";
import UserAvatar from "@/components/UserAvatar.vue";
import StatusBadge from "@/components/StatusBadge.vue";
import Modal from "@/components/Modal.vue";
import UserForm from "@/components/UserForm.vue";
import userService from "@/services/userService";
import type { User } from "@/services/userService";

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

// Get unique roles from users
const availableRoles = computed(() => {
  if (!users.value.length) return [];
  const roles = new Set(users.value.map((user) => user.role));
  return Array.from(roles) as string[];
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

const filteredUsers = computed(() => {
  if (!users.value.length) return [];
  
  return users.value.filter((user) => {
    // Text search (case insensitive)
    const searchLower = searchQuery.value.toLowerCase();
    const matchesSearch =
      searchQuery.value === "" ||
      user.name.toLowerCase().includes(searchLower) ||
      user.email.toLowerCase().includes(searchLower) ||
      user.role.toLowerCase().includes(searchLower) ||
      (user.department && user.department.toLowerCase().includes(searchLower));

    // Role filter
    const matchesRole =
      roleFilter.value === "all" || 
      user.role === roleFilter.value;

    return matchesSearch && matchesRole;
  }).sort((a, b) => {
    // Sort by the selected field
    let aValue: any;
    let bValue: any;

    switch (sortField.value) {
      case "id":
        aValue = a.id;
        bValue = b.id;
        break;
      case "name":
        aValue = a.name.toLowerCase();
        bValue = b.name.toLowerCase();
        break;
      case "role":
        aValue = a.role.toLowerCase();
        bValue = b.role.toLowerCase();
        break;
      case "department":
        aValue = (a.department || "").toLowerCase();
        bValue = (b.department || "").toLowerCase();
        break;
      default:
        aValue = a.id;
        bValue = b.id;
    }

    // Apply sort direction
    const direction = sortDirection.value === "asc" ? 1 : -1;
    
    // Handle string comparison
    if (typeof aValue === "string" && typeof bValue === "string") {
      return aValue.localeCompare(bValue) * direction;
    }
    
    // Handle number comparison
    return (aValue - bValue) * direction;
  });
});

const navigateToUser = (uuid: string) => {
  router.push(`/users/${uuid}`);
};

const fetchUsers = async () => {
  loading.value = true;
  error.value = null;
  
  try {
    const fetchedUsers = await userService.getUsers();
    
    // Transform backend users to UI users with additional properties
    users.value = fetchedUsers.map(user => ({
      ...user,
      department: "IT Support", // Default department (could be added to backend later)
    }));
    
    console.log('Users loaded:', users.value);
  } catch (err) {
    console.error('Error fetching users:', err);
    error.value = 'Failed to load users. Please try again later.';
  } finally {
    loading.value = false;
  }
};

// Reset all filters
const resetFilters = () => {
  searchQuery.value = "";
  roleFilter.value = "all";
};

// Handle filter updates from BaseListView
const handleFilterUpdate = (name: string, value: string) => {
  if (name === 'role') {
    roleFilter.value = value;
  }
};

// Add function to handle user creation
const handleCreateUser = async (userData: { name: string; email: string; role: string }) => {
  isCreatingUser.value = true;
  createUserError.value = null;
  
  try {
    await userService.createUser(userData);
    showAddUserModal.value = false;
    // Refresh the user list
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
    const currentIndex = filteredUsers.value.findIndex(
      (user) => user.uuid === userUuid
    );
    const lastIndex = filteredUsers.value.findIndex(
      (user) => user.uuid === lastSelectedUserUuid.value
    );

    if (currentIndex !== -1 && lastIndex !== -1) {
      const startIndex = Math.min(currentIndex, lastIndex);
      const endIndex = Math.max(currentIndex, lastIndex);

      const usersToSelect = filteredUsers.value
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
    selectedUsers.value = filteredUsers.value.map((user) => user.uuid);
  } 
  // If unchecking, clear all selections
  else {
    selectedUsers.value = [];
  }
  
  // Reset last selected user
  lastSelectedUserUuid.value = null;
};

// Add function to toggle sort
const toggleSort = (field: string) => {
  if (sortField.value === field) {
    // Toggle direction if clicking the same field
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    // Set new field and default to ascending
    sortField.value = field;
    sortDirection.value = "asc";
  }
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
};

onMounted(() => {
  fetchUsers();
});
</script>

<template>
  <BaseListView
    title="Users"
    :search-query="searchQuery"
    :is-loading="loading"
    :is-empty="filteredUsers.length === 0"
    :error="error"
    :filters="filterOptions"
    :results-count="filteredUsers.length"
    :sort-field="sortField"
    :sort-direction="sortDirection"
    :columns="columns"
    :selected-items="selectedUsers"
    :visible-items="filteredUsers"
    :item-id-field="'uuid'"
    :enable-selection="true"
    @update:search-query="value => searchQuery = value"
    @update:filter="handleFilterUpdate"
    @update:sort="handleSortUpdate"
    @toggle-selection="toggleSelection"
    @toggle-all="toggleAllUsers"
    @reset-filters="resetFilters"
    @add="showAddUserModal = true"
    @retry="fetchUsers"
  >
    <div class="min-w-[800px]">
      <div
        v-for="user in filteredUsers"
        :key="user.uuid"
        @click="navigateToUser(user.uuid)"
        class="flex border-b border-slate-800 text-sm text-gray-200 hover:bg-slate-800/50 transition-colors cursor-pointer gap-2"
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
  </BaseListView>
</template>

<style scoped>
/* Custom scrollbar styling moved to BaseListView */
</style>
