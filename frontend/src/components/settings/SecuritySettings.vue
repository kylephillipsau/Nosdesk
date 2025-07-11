<script setup lang="ts">
import { ref, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';

// Get current user info
const authStore = useAuthStore();

// Form state
const currentPassword = ref('');
const newPassword = ref('');
const confirmPassword = ref('');
const loading = ref(false);

// Emits for notifications
const emit = defineEmits<{
  (e: 'success', message: string): void;
  (e: 'error', message: string): void;
}>();

// Validation
const passwordsMatch = computed(() => {
  return newPassword.value === confirmPassword.value;
});

const isFormValid = computed(() => {
  return currentPassword.value.length > 0 &&
         newPassword.value.length >= 8 &&
         passwordsMatch.value;
});

// Password change function
const changePassword = async () => {
  if (!isFormValid.value) {
    emit('error', 'Please fill in all fields correctly');
    return;
  }

  loading.value = true;

  try {
    // TODO: Implement password change API call
    // await authService.changePassword(currentPassword.value, newPassword.value);
    
    // Reset form
    currentPassword.value = '';
    newPassword.value = '';
    confirmPassword.value = '';
    
    emit('success', 'Password changed successfully');
  } catch (err) {
    emit('error', 'Failed to change password. Please check your current password.');
    console.error('Error changing password:', err);
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <h2 class="text-lg font-medium text-white">Security</h2>
      <p class="text-sm text-slate-400 mt-1">Change your password and manage account security</p>
    </div>
    
    <div class="p-6">
      <form @submit.prevent="changePassword" class="flex flex-col gap-4">
        <!-- Hidden username field for accessibility and password managers -->
        <input
          type="email"
          :value="authStore.user?.email || ''"
          autocomplete="username"
          class="sr-only"
          tabindex="-1"
          readonly
        />
        
        <!-- Current Password -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-slate-400 uppercase tracking-wide">Current Password</label>
          <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
            <input
              v-model="currentPassword"
              type="password"
              autocomplete="current-password"
              class="w-full px-4 py-2 bg-transparent text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
              placeholder="Enter your current password"
              required
            />
          </div>
        </div>

        <!-- New Password -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-slate-400 uppercase tracking-wide">New Password</label>
          <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
            <input
              v-model="newPassword"
              type="password"
              autocomplete="new-password"
              class="w-full px-4 py-2 bg-transparent text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
              placeholder="Enter your new password"
              minlength="8"
              required
            />
          </div>
          <p class="text-xs text-slate-400">Password must be at least 8 characters long</p>
        </div>

        <!-- Confirm New Password -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-slate-400 uppercase tracking-wide">Confirm New Password</label>
          <div class="bg-slate-700/50 rounded-lg border border-slate-600/30">
            <input
              v-model="confirmPassword"
              type="password"
              autocomplete="new-password"
              class="w-full px-4 py-2 bg-transparent text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
              placeholder="Confirm your new password"
              required
            />
          </div>
          <p v-if="confirmPassword && !passwordsMatch" class="text-xs text-red-400">
            Passwords do not match
          </p>
        </div>

        <!-- Submit Button -->
        <div class="pt-4">
          <button
            type="submit"
            :disabled="!isFormValid || loading"
            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center"
          >
            <span v-if="loading" class="animate-spin h-4 w-4 mr-2">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 004 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            </span>
            Change Password
          </button>
        </div>
      </form>
    </div>
  </div>
</template> 