<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import UserAvatar from '@/components/UserAvatar.vue';
import { useDataStore } from '@/stores/dataStore';

const props = defineProps<{
  modelValue: string; // The selected user's ID or name (bound via v-model)
  placeholder?: string;
  type: 'requester' | 'assignee'; // To differentiate between requester and assignee for emits
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void; // For v-model
}>();

// Use data store for user lookups
const dataStore = useDataStore();

// State for search functionality
const inputValue = ref('');
const isDropdownOpen = ref(false);
const isInputFocused = ref(false);
const searchResults = ref<{ id: string; name: string; email: string }[]>([]);
const isSearching = ref(false);
const searchError = ref<string | null>(null);
const selectedUser = ref<{ id: string; name: string; email: string } | null>(null);

// Debounce timer for search
let searchTimeout: number | null = null;

// Find user info by UUID
const getUserInfo = async (id: string) => {
  if (!id) return { name: '', user: null };
  
  // If we have the selected user cached, use it
  if (selectedUser.value && selectedUser.value.id === id) {
    return { name: selectedUser.value.name, user: selectedUser.value };
  }
  
  // Try to get from data store cache
  const cachedName = dataStore.getUserName(id);
  if (cachedName) {
    const user = { id, name: cachedName, email: '' };
    selectedUser.value = user;
    return { name: cachedName, user };
  }
  
  // If it's a UUID pattern, try to fetch from store
  if (id.length === 36 && id.includes('-')) {
    try {
      const user = await dataStore.getUserByUuid(id);
      if (user) {
        const userInfo = { id: user.uuid, name: user.name, email: user.email };
        selectedUser.value = userInfo;
        return { name: user.name, user: userInfo };
      }
    } catch (error) {
      console.error('Error fetching user data:', error);
    }
    return { name: 'Loading...', user: null };
  }
  
  // Fallback to the ID itself
  return { name: id, user: null };
};

// Initialize the component with the current modelValue
const initializeUser = async () => {
  if (props.modelValue) {
    const info = await getUserInfo(props.modelValue);
    inputValue.value = info.name;
  }
};

// Search users via API
const searchUsers = async (query: string) => {
  if (!query || query.length < 2) {
    searchResults.value = [];
    return;
  }
  
  try {
    isSearching.value = true;
    searchError.value = null;
    
    // Use the data store to search users
    const response = await dataStore.getPaginatedUsers({
      page: 1,
      pageSize: 50, // Limit results for dropdown
      search: query,
      sortField: 'name',
      sortDirection: 'asc'
    });
    
    // Transform users to the format expected by the component
    searchResults.value = response.data.map(user => ({
      id: user.uuid,
      name: user.name,
      email: user.email
    }));
  } catch (error) {
    console.error('Error searching users:', error);
    searchError.value = 'Failed to search users';
    searchResults.value = [];
  } finally {
    isSearching.value = false;
  }
};

// Debounced search function
const debouncedSearch = (query: string) => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  
  searchTimeout = setTimeout(() => {
    searchUsers(query);
  }, 300); // 300ms delay
};

// Watch for modelValue changes
watch(() => props.modelValue, async (newValue) => {
  if (newValue === null || newValue === undefined || newValue === '') {
    inputValue.value = '';
    selectedUser.value = null;
  } else {
    const info = await getUserInfo(newValue);
    inputValue.value = info.name;
  }
});

// Handle user selection
const selectUser = (user: { id: string; name: string; email: string }) => {
  selectedUser.value = user;
  inputValue.value = user.name;
  console.log(`UserSelection: Emitting update:modelValue with ID: ${user.id}`);
  emit('update:modelValue', user.id);
  isDropdownOpen.value = false;
  searchResults.value = [];
};

// Clear selection
const clearSelection = () => {
  selectedUser.value = null;
  inputValue.value = '';
  emit('update:modelValue', '');
  isDropdownOpen.value = false;
  searchResults.value = [];
};

// Handle input focus
const handleFocus = (event: Event) => {
  isInputFocused.value = true;
  isDropdownOpen.value = true;
  
  // If there's no current search and no selected user, don't search yet
  if (!inputValue.value || inputValue.value === selectedUser.value?.name) {
    // Clear the input to allow fresh search
    if (selectedUser.value) {
      inputValue.value = '';
    }
  }
  
  // Select all text when input receives focus
  const input = event.target as HTMLInputElement;
  setTimeout(() => input.select(), 0);
};

// Handle input changes
const handleInput = () => {
  isDropdownOpen.value = true;
  
  // If user clears the input, clear the selection
  if (inputValue.value === '') {
    if (selectedUser.value) {
      emit('update:modelValue', '');
      selectedUser.value = null;
    }
    searchResults.value = [];
    return;
  }
  
  // If the input doesn't match the selected user, start searching
  if (!selectedUser.value || inputValue.value !== selectedUser.value.name) {
    debouncedSearch(inputValue.value);
  }
};

// Handle input blur
const handleBlur = () => {
  isInputFocused.value = false;
  
  // Restore the selected user's name if input was changed but no new selection made
  if (selectedUser.value && inputValue.value !== selectedUser.value.name) {
    inputValue.value = selectedUser.value.name;
  }
  
  // Delay closing to allow click events on dropdown items
  setTimeout(() => {
    if (!isInputFocused.value) {
      isDropdownOpen.value = false;
      searchResults.value = [];
    }
  }, 200);
};

// Handle clicks outside
const handleClickOutside = (event: MouseEvent) => {
  const dropdown = document.querySelector('.user-autocomplete-dropdown');
  const input = document.querySelector('.user-autocomplete-input');
  
  if (dropdown && !dropdown.contains(event.target as Node) && 
      input && !input.contains(event.target as Node)) {
    if (!isInputFocused.value) {
      isDropdownOpen.value = false;
      searchResults.value = [];
    }
  }
};

// Cleanup
onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  initializeUser();
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
});
</script>

<template>
  <div class="relative">
    <div class="flex items-center gap-2">
      <!-- Show UserAvatar when a user is selected and dropdown is not open -->
      <UserAvatar
        v-if="inputValue && inputValue !== 'Loading...' && !isDropdownOpen"
        :name="props.modelValue"
        :showName="false"
        class="w-8 h-8"
      />
      
      <!-- Input container with clear button -->
      <div class="relative flex-1">
        <input
          type="text"
          v-model="inputValue"
          @focus="handleFocus"
          @input="handleInput"
          @blur="handleBlur"
          :placeholder="placeholder || 'Search or select user...'"
          class="user-autocomplete-input px-2 py-1 pr-8 text-sm rounded bg-slate-600 text-slate-200 w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        
        <!-- Clear button -->
        <button
          v-if="inputValue && inputValue !== 'Loading...'"
          @click="clearSelection"
          class="absolute right-2 top-1/2 transform -translate-y-1/2 text-slate-400 hover:text-slate-200 transition-colors"
          type="button"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        
        <!-- Loading indicator -->
        <div
          v-if="inputValue === 'Loading...'"
          class="absolute right-2 top-1/2 transform -translate-y-1/2"
        >
          <svg class="w-4 h-4 animate-spin text-slate-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
      </div>
    </div>
    
    <!-- Dropdown with users -->
    <ul
      v-if="isDropdownOpen && searchResults.length > 0"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto overflow-x-visible z-50 border border-slate-700"
    >
      <li
        v-for="user in searchResults"
        :key="user.id"
        @click="selectUser(user)"
        class="p-3 cursor-pointer hover:bg-slate-700 transition-colors text-slate-200 flex items-center gap-3 border-b border-slate-700/50 last:border-b-0"
      >
        <UserAvatar :name="user.id" :showName="false" size="sm"/>
        <div class="flex-1 min-w-0">
          <div class="flex flex-col">
            <span class="font-medium text-sm truncate">{{ user.name }}</span>
            <span class="text-xs text-slate-400 truncate">{{ user.email }}</span>
          </div>
        </div>
      </li>
    </ul>
    
    <!-- Loading state -->
    <ul
      v-if="isDropdownOpen && isSearching"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto z-50 border border-slate-700"
    >
      <li class="p-3 text-slate-400 text-center text-sm flex items-center justify-center gap-2">
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Searching users...
      </li>
    </ul>
    
    <!-- Search error message -->
    <ul
      v-if="isDropdownOpen && searchError"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto z-50 border border-slate-700"
    >
      <li class="p-3 text-red-400 text-center text-sm">
        {{ searchError }}
      </li>
    </ul>
    
    <!-- No users found message -->
    <ul
      v-if="isDropdownOpen && !isSearching && searchResults.length === 0 && inputValue && inputValue.length >= 2 && !searchError"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto z-50 border border-slate-700"
    >
      <li class="p-3 text-slate-400 text-center text-sm">
        No users found for "{{ inputValue }}"
      </li>
    </ul>
    
    <!-- Search prompt -->
    <ul
      v-if="isDropdownOpen && !isSearching && inputValue && inputValue.length < 2 && !selectedUser"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto z-50 border border-slate-700"
    >
      <li class="p-3 text-slate-400 text-center text-sm">
        Type at least 2 characters to search
      </li>
    </ul>
  </div>
</template>