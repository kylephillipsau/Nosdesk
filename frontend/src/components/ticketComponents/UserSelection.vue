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

// Add reference for input positioning
const inputRef = ref<HTMLElement | null>(null);
const containerRef = ref<HTMLElement | null>(null);

// Debounce timer for search
let searchTimeout: number | null = null;

// Computed position for the dropdown
const dropdownPosition = computed(() => {
  if (!containerRef.value || !isDropdownOpen.value) {
    return { top: '0px', left: '0px', width: '0px' }
  }
  
  const rect = containerRef.value.getBoundingClientRect()
  return {
    top: `${rect.bottom + window.scrollY + 4}px`,
    left: `${rect.left + window.scrollX}px`,
    width: `${rect.width}px`
  }
})

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
  console.log(`UserSelection: Previous modelValue was: ${props.modelValue}`);
  emit('update:modelValue', user.id);
  isDropdownOpen.value = false;
  searchResults.value = [];
};

// Clear selection
const clearSelection = () => {
  selectedUser.value = null;
  inputValue.value = '';
  console.log(`UserSelection: Clearing selection, emitting empty string`);
  console.log(`UserSelection: Previous modelValue was: ${props.modelValue}`);
  emit('update:modelValue', '');
  isDropdownOpen.value = false;
  searchResults.value = [];
};

// Handle input focus
const handleFocus = (event: Event) => {
  isInputFocused.value = true;
  isDropdownOpen.value = true;
  
  // Select all text when input receives focus to allow easy replacement
  const input = event.target as HTMLInputElement;
  setTimeout(() => input.select(), 0);
  
  // If we have a selected user, search for users matching the current name
  // This allows finding other users with the same name
  if (selectedUser.value && inputValue.value) {
    debouncedSearch(inputValue.value);
  }
};

// Handle input changes
const handleInput = () => {
  isDropdownOpen.value = true;
  
  // If user clears the input, clear the selection
  if (inputValue.value === '') {
    if (selectedUser.value) {
      console.log(`UserSelection: Input cleared, emitting empty string`);
      emit('update:modelValue', '');
      selectedUser.value = null;
    }
    searchResults.value = [];
    return;
  }
  
  // Always search when input changes (even if it matches current user's name)
  // This allows selecting different users with the same name
  debouncedSearch(inputValue.value);
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
  
  if (containerRef.value && !containerRef.value.contains(event.target as Node) && 
      dropdown && !dropdown.contains(event.target as Node)) {
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
  <div ref="containerRef" class="relative w-full">
    <!-- User Selection Container -->
    <div class="flex items-center gap-2 px-2.5 py-1.5 min-h-[36px]">
      <!-- Avatar Space - Always present for consistent alignment -->
      <div class="flex-shrink-0 w-6 h-6 flex items-center justify-center">
        <UserAvatar
          v-if="selectedUser && !isDropdownOpen"
          :name="selectedUser.id"
          :showName="false"
          size="sm"
        />
        <!-- Placeholder when no user selected -->
        <div
          v-else
          class="w-6 h-6 rounded-full bg-slate-700/30 border border-slate-600/30 flex items-center justify-center transition-all duration-200"
          :class="{ 'border-slate-500/50': isInputFocused }"
        >
          <svg class="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        </div>
      </div>
      
      <!-- Input Container -->
      <div class="flex-1 relative">
        <input
          ref="inputRef"
          type="text"
          v-model="inputValue"
          @focus="handleFocus"
          @input="handleInput"
          @blur="handleBlur"
          :placeholder="placeholder || 'Select user...'"
          class="w-full bg-transparent text-slate-200 placeholder-slate-400 focus:outline-none text-sm transition-all duration-200 leading-tight"
          :class="{
            'text-slate-300': selectedUser && !isDropdownOpen,
            'text-slate-200': !selectedUser || isDropdownOpen
          }"
        />
      </div>
      
      <!-- Action Buttons -->
      <div class="flex items-center gap-1 flex-shrink-0">
        <!-- Clear button -->
        <button
          v-if="selectedUser"
          @click.stop="clearSelection"
          class="p-1 text-slate-400 hover:text-slate-200 hover:bg-slate-600/30 rounded transition-all duration-200 group"
          type="button"
          title="Clear selection"
        >
          <svg class="w-3 h-3 group-hover:scale-110 transition-transform duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        
        <!-- Search/Loading indicator -->
        <div class="flex items-center justify-center w-5 h-5">
          <div v-if="isSearching" class="w-3 h-3 border-2 border-slate-400 border-t-transparent rounded-full animate-spin"></div>
          <svg v-else-if="isDropdownOpen" class="w-3 h-3 text-slate-400 transition-transform duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
          <svg v-else class="w-3 h-3 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
      </div>
    </div>
    
    <!-- Dropdown with users -->
    <Teleport to="body">
      <div
        v-if="isDropdownOpen && searchResults.length > 0 && containerRef"
        class="user-autocomplete-dropdown fixed bg-slate-800 rounded-lg shadow-xl border border-slate-700/50 min-w-max backdrop-blur-sm z-[9999] overflow-hidden"
        :style="dropdownPosition"
      >
        <div class="py-1 max-h-56 overflow-y-auto">
          <button
            v-for="user in searchResults.slice(0, 8)"
            :key="user.id"
            @click="selectUser(user)"
            class="w-full px-2.5 py-1.5 text-left flex items-center gap-2.5 hover:bg-slate-700/50 transition-all duration-150 group"
            :class="{
              'bg-slate-700/30': selectedUser?.id === user.id
            }"
          >
            <UserAvatar 
              :name="user.id" 
              :showName="false" 
              size="sm"
            />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-slate-200 truncate">{{ user.name }}</div>
              <div class="text-xs text-slate-400 truncate">{{ user.email }}</div>
            </div>
            <svg 
              v-if="selectedUser?.id === user.id" 
              class="w-3 h-3 text-blue-400 flex-shrink-0" 
              fill="currentColor" 
              viewBox="0 0 20 20"
            >
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </div>
    </Teleport>
    
    <!-- Loading state -->
    <Teleport to="body">
      <div
        v-if="isDropdownOpen && isSearching && containerRef"
        class="user-autocomplete-dropdown fixed bg-slate-800 rounded-lg shadow-xl border border-slate-700/50 min-w-max backdrop-blur-sm z-[9999]"
        :style="dropdownPosition"
      >
        <div class="p-3 flex items-center justify-center gap-2 text-slate-400">
          <div class="w-4 h-4 border-2 border-slate-400 border-t-transparent rounded-full animate-spin"></div>
          <span class="text-sm">Searching...</span>
        </div>
      </div>
    </Teleport>
    
    <!-- Search error message -->
    <Teleport to="body">
      <div
        v-if="isDropdownOpen && searchError && containerRef"
        class="user-autocomplete-dropdown fixed bg-slate-800 rounded-lg shadow-xl border border-slate-700/50 min-w-max backdrop-blur-sm z-[9999]"
        :style="dropdownPosition"
      >
        <div class="p-3 text-center">
          <div class="text-red-400 text-sm">{{ searchError }}</div>
        </div>
      </div>
    </Teleport>
    
    <!-- No users found message -->
    <Teleport to="body">
      <div
        v-if="isDropdownOpen && !isSearching && searchResults.length === 0 && inputValue && inputValue.length >= 2 && !searchError && containerRef"
        class="user-autocomplete-dropdown fixed bg-slate-800 rounded-lg shadow-xl border border-slate-700/50 min-w-max backdrop-blur-sm z-[9999]"
        :style="dropdownPosition"
      >
        <div class="p-3 text-center text-slate-400">
          <div class="text-sm">No users found</div>
        </div>
      </div>
    </Teleport>
    
    <!-- Search prompt -->
    <Teleport to="body">
      <div
        v-if="isDropdownOpen && !isSearching && inputValue && inputValue.length < 2 && !selectedUser && containerRef"
        class="user-autocomplete-dropdown fixed bg-slate-800 rounded-lg shadow-xl border border-slate-700/50 min-w-max backdrop-blur-sm z-[9999]"
        :style="dropdownPosition"
      >
        <div class="p-3 text-center text-slate-400">
          <div class="text-sm">Type to search</div>
        </div>
      </div>
    </Teleport>
  </div>
</template>