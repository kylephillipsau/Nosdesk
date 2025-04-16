<!-- UserForm.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';

interface UserFormData {
  name: string;
  email: string;
  role: string;
}

const props = defineProps<{
  initialData?: UserFormData;
  isEdit?: boolean;
}>();

const emit = defineEmits<{
  (e: 'submit', data: UserFormData): void;
  (e: 'cancel'): void;
}>();

const formData = ref<UserFormData>({
  name: '',
  email: '',
  role: 'user'
});

// If initialData is provided, use it to populate the form
watch(() => props.initialData, (newData) => {
  if (newData) {
    formData.value = { ...newData };
  }
}, { immediate: true });

const roleOptions = [
  { value: 'user', label: 'User' },
  { value: 'technician', label: 'Technician' },
  { value: 'admin', label: 'Admin' }
];

const submitForm = () => {
  emit('submit', formData.value);
};
</script>

<template>
  <form @submit.prevent="submitForm" class="space-y-4">
    <div>
      <label for="name" class="block text-sm font-medium text-slate-300 mb-1">Name</label>
      <input
        id="name"
        v-model="formData.name"
        type="text"
        required
        class="w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="Enter user name"
      />
    </div>
    
    <div>
      <label for="email" class="block text-sm font-medium text-slate-300 mb-1">Email</label>
      <input
        id="email"
        v-model="formData.email"
        type="email"
        required
        class="w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="Enter email address"
      />
    </div>
    
    <div>
      <label for="role" class="block text-sm font-medium text-slate-300 mb-1">Role</label>
      <select
        id="role"
        v-model="formData.role"
        required
        class="w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option v-for="option in roleOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>
    
    <div class="flex justify-end space-x-3 pt-4">
      <button
        type="button"
        @click="emit('cancel')"
        class="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
      >
        Cancel
      </button>
      <button
        type="submit"
        class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
      >
        {{ props.isEdit ? 'Update' : 'Create' }} User
      </button>
    </div>
  </form>
</template> 