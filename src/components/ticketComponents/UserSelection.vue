<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import UserAvatar from '@/components/UserAvatar.vue'; // Import the UserAvatar component

const props = defineProps<{
  modelValue: string; // The selected user's ID or name (bound via v-model)
  users: {
    id: string;
    name: string;
    email: string;
  }[]; // Removed 'role' from the users structure
  placeholder?: string;
  type: 'requester' | 'assignee'; // To differentiate between requester and assignee for emits
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void; // For v-model
}>();

const inputValue = ref(props.modelValue || '');
const isDropdownOpen = ref(false);

// Track if the input is currently focused to prevent premature closing
const isInputFocused = ref(false);

// Filtered users based on input
const filteredUsers = computed(() => {
  const query = inputValue.value.toLowerCase();
  return props.users.filter(user =>
    user.name.toLowerCase().includes(query) ||
    user.email.toLowerCase().includes(query)
  );
});

// Handle user selection
const selectUser = (userId: string) => {
  const user = props.users.find(u => u.id === userId);
  if (user) {
    inputValue.value = user.name; // Display the user's name in the input
    emit('update:modelValue', userId); // Emit the user ID
  }
  isDropdownOpen.value = false;
};

// Open dropdown on focus or input, but only if there’s a change or focus
const handleFocus = () => {
  isInputFocused.value = true;
  isDropdownOpen.value = true;
};

const handleInput = () => {
  isDropdownOpen.value = true;
};

// Close dropdown when clicking outside, but only if the input isn’t focused
const handleClickOutside = (event: MouseEvent) => {
  const dropdown = document.querySelector('.user-autocomplete-dropdown');
  const input = document.querySelector('.user-autocomplete-input');
  
  // Only close if the click is outside both the dropdown and input, and the input isn’t focused
  if (dropdown && !dropdown.contains(event.target as Node) && input && !input.contains(event.target as Node)) {
    if (!isInputFocused.value) {
      isDropdownOpen.value = false;
    }
  }
};

// Handle blur to track when input loses focus
const handleBlur = () => {
  isInputFocused.value = false;
  // Optionally delay closing to allow click events on dropdown items
  setTimeout(() => {
    if (!isInputFocused.value) {
      isDropdownOpen.value = false;
    }
  }, 200); // Small delay to allow clicks on dropdown items
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div class="relative">
    <div class="flex items-center gap-2">
      <!-- Show UserAvatar when a user is selected (inputValue has a name) and dropdown is not open -->
      <UserAvatar
        v-if="inputValue && !isDropdownOpen"
        :name="inputValue"
        :showName="false"
        class="w-8 h-8"
      />
      <input
        type="text"
        v-model="inputValue"
        @focus="handleFocus"
        @input="handleInput"
        @blur="handleBlur"
        :placeholder="placeholder || 'Search or select user...'"
        class="user-autocomplete-input px-2 py-1 text-sm rounded bg-slate-600 text-slate-200 w-full focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>
    <ul
      v-if="isDropdownOpen && filteredUsers.length > 0"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto overflow-x-visible z-50"
    >
      <li
        v-for="user in filteredUsers"
        :key="user.id"
        @click="selectUser(user.id)"
        class="p-2 rounded-md cursor-pointer hover:bg-slate-700 transition-colors text-slate-200 flex items-center gap-2"
      >
        <UserAvatar :name="user.name" :showName="false" class="w-8 h-8" />
        <div class="flex-1">
          <div class="flex flex-col justify-between items-start">
            <span class="font-medium">{{ user.name }}</span>
            <span class="text-xs text-slate-500">{{ user.email }}</span>
          </div>
        </div>
      </li>
    </ul>
    <ul
      v-if="isDropdownOpen && filteredUsers.length === 0"
      class="user-autocomplete-dropdown absolute left-0 right-0 mt-1 bg-slate-800 rounded-lg shadow-xl max-h-60 overflow-y-auto z-50"
    >
      <li class="p-2 text-slate-400 text-center">No users found</li>
    </ul>
  </div>
</template>