<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAuthStore } from '@/stores/auth';
import BackButton from '@/components/common/BackButton.vue';
import UserAvatar from '@/components/UserAvatar.vue';
import axios from 'axios';

const authStore = useAuthStore();
const loading = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const showDeveloperSettings = ref(false);

// Form data
const formData = ref({
  name: authStore.user?.name || '',
  email: authStore.user?.email || '',
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
});

const updateProfile = async () => {
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // TODO: Implement profile update logic
    // await userService.updateProfile(formData.value);
    successMessage.value = 'Profile updated successfully';
  } catch (err) {
    error.value = 'Failed to update profile';
    console.error('Error updating profile:', err);
  } finally {
    loading.value = false;
  }
};

const updatePassword = async () => {
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  // Validation
  if (formData.value.newPassword !== formData.value.confirmPassword) {
    error.value = 'New password and confirmation do not match';
    loading.value = false;
    return;
  }

  if (!formData.value.currentPassword || !formData.value.newPassword) {
    error.value = 'All password fields are required';
    loading.value = false;
    return;
  }

  try {
    const response = await axios.post('/api/auth/change-password', {
      current_password: formData.value.currentPassword,
      new_password: formData.value.newPassword
    });

    successMessage.value = 'Password updated successfully';
    
    // Clear password fields
    formData.value.currentPassword = '';
    formData.value.newPassword = '';
    formData.value.confirmPassword = '';
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to update password';
    console.error('Error updating password:', err);
  } finally {
    loading.value = false;
  }
};

const handleAvatarClick = () => {
  fileInput.value?.click();
};

const handleFileChange = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files?.length) return;

  const file = input.files[0];
  if (!file.type.startsWith('image/')) {
    error.value = 'Please select an image file';
    return;
  }

  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // TODO: Implement avatar upload logic
    // const formData = new FormData();
    // formData.append('avatar', file);
    // await userService.updateAvatar(formData);
    successMessage.value = 'Avatar updated successfully';
  } catch (err) {
    error.value = 'Failed to update avatar';
    console.error('Error updating avatar:', err);
  } finally {
    loading.value = false;
  }
};

const toggleAdminRole = () => {
  const success = authStore.setAdminRole(!authStore.isAdmin);
  if (success) {
    successMessage.value = `Admin role ${authStore.isAdmin ? 'enabled' : 'disabled'} successfully`;
  }
};

onMounted(() => {
  document.title = 'Profile Settings | nosDesk';
});
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div class="pt-4 px-6 flex justify-between items-center">
      <BackButton fallbackRoute="/" label="Back to Dashboard" />
    </div>
    
    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
      <!-- Grid Container -->
      <div class="grid-container">
        <!-- User Info Area -->
        <div class="info-area flex flex-col gap-4">
          <!-- User Profile Header -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <div class="flex items-start gap-4">
              <!-- Avatar Container with hover effect -->
              <div 
                class="w-32 h-32 flex-shrink-0 relative rounded-lg overflow-hidden group cursor-pointer"
                @click="handleAvatarClick"
              >
                <UserAvatar
                  :name="authStore.user?.name || ''"
                  size="full"
                  :showName="false"
                  :clickable="false"
                  class="w-full h-full"
                />
                <!-- Hover Overlay -->
                <div class="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                  <div class="text-white flex flex-col items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <span class="text-sm">Change Photo</span>
                  </div>
                </div>
                <!-- Hidden file input -->
                <input
                  ref="fileInput"
                  type="file"
                  accept="image/*"
                  class="hidden"
                  @change="handleFileChange"
                />
              </div>
              <div class="flex-1 min-w-0">
                <h1 class="text-2xl font-semibold text-white truncate">{{ authStore.user?.name }}</h1>
                <p class="text-slate-400 truncate">{{ authStore.user?.email }}</p>
                <div class="mt-2 flex items-center space-x-4">
                  <span class="text-sm text-slate-400">{{ authStore.user?.role || 'User' }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Settings Form -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <h2 class="text-lg font-medium text-white mb-6">Profile Information</h2>
            
            <form @submit.prevent="updateProfile" class="flex flex-col gap-2">
              <!-- Success/Error messages -->
              <div v-if="successMessage" class="p-4 bg-green-900/50 text-green-400 rounded-lg">
                {{ successMessage }}
              </div>
              <div v-if="error" class="p-4 bg-red-900/50 text-red-400 rounded-lg">
                {{ error }}
              </div>

              <!-- Name -->
              <div class="flex flex-col gap-1">
                <label for="name" class="text-sm font-medium text-slate-300">Name</label>
                <input
                  id="name"
                  v-model="formData.name"
                  type="text"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Your name"
                />
              </div>

              <!-- Email -->
              <div class="flex flex-col gap-1">
                <label for="email" class="text-sm font-medium text-slate-300">Email</label>
                <input
                  id="email"
                  v-model="formData.email"
                  type="email"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="your.email@example.com"
                />
              </div>

              <!-- Submit button for profile info -->
              <div class="flex justify-end pt-2">
                <button
                  type="submit"
                  :disabled="loading"
                  class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                >
                  <span v-if="loading" class="animate-spin h-4 w-4">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  </span>
                  {{ loading ? 'Saving...' : 'Save Changes' }}
                </button>
              </div>
            </form>
          </div>
        </div>

        <!-- Security Area -->
        <div class="security-area flex flex-col gap-4">
          <!-- Password Change Section -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <h2 class="text-lg font-medium text-white mb-6">Security Settings</h2>
            
            <form @submit.prevent="updatePassword" class="flex flex-col gap-6">
              <!-- Success/Error messages -->
              <div v-if="successMessage" class="p-4 bg-green-900/50 text-green-400 rounded-lg">
                {{ successMessage }}
              </div>
              <div v-if="error" class="p-4 bg-red-900/50 text-red-400 rounded-lg">
                {{ error }}
              </div>
              
              <!-- Current Password -->
              <div class="flex flex-col gap-2">
                <label for="currentPassword" class="text-sm font-medium text-slate-300">Current Password</label>
                <input
                  id="currentPassword"
                  v-model="formData.currentPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Enter your current password"
                />
              </div>

              <!-- New Password -->
              <div class="flex flex-col gap-2">
                <label for="newPassword" class="text-sm font-medium text-slate-300">New Password</label>
                <input
                  id="newPassword"
                  v-model="formData.newPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Enter new password"
                />
              </div>

              <!-- Confirm Password -->
              <div class="flex flex-col gap-2">
                <label for="confirmPassword" class="text-sm font-medium text-slate-300">Confirm New Password</label>
                <input
                  id="confirmPassword"
                  v-model="formData.confirmPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Confirm new password"
                />
              </div>

              <!-- Submit button for password change -->
              <div class="flex justify-end pt-2">
                <button
                  type="submit"
                  :disabled="loading"
                  class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                >
                  <span v-if="loading" class="animate-spin h-4 w-4">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  </span>
                  {{ loading ? 'Updating...' : 'Update Password' }}
                </button>
              </div>
            </form>
          </div>
          
          <!-- Developer Options -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <div class="flex justify-between items-center mb-4">
              <h2 class="text-lg font-medium text-white">Developer Settings</h2>
              <button 
                @click="showDeveloperSettings = !showDeveloperSettings"
                class="text-sm text-slate-400 hover:text-white"
              >
                {{ showDeveloperSettings ? 'Hide' : 'Show' }}
              </button>
            </div>
            
            <div v-if="showDeveloperSettings" class="border-t border-slate-700 pt-4">
              <div class="text-sm text-slate-400 mb-4">
                These settings are for development purposes only. They will be removed in production.
              </div>
              
              <!-- Admin Role Toggle -->
              <div class="flex justify-between items-center py-2">
                <div>
                  <div class="text-white font-medium">Admin Role</div>
                  <div class="text-sm text-slate-400">
                    Enable admin privileges for testing admin-only features
                  </div>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input 
                    type="checkbox" 
                    :checked="authStore.isAdmin"
                    @change="toggleAdminRole"
                    class="sr-only peer"
                  >
                  <div class="w-11 h-6 bg-slate-700 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-slate-400 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 peer-checked:after:bg-white"></div>
                </label>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-template-areas: "info" "security";
  gap: 1rem;

  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas: "info security";
  }
}

.info-area {
  grid-area: info;
}

.security-area {
  grid-area: security;
}
</style> 