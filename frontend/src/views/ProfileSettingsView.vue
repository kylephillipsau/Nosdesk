<script setup lang="ts">
import { ref } from 'vue';
import BackButton from '@/components/common/BackButton.vue';
import {
  UserProfileCard,
  AppearanceSettings,
  NotificationSettings,
  SecuritySettings,
  MFASettings,
  AuthMethodsSettings
} from '@/components/settings';

// Global state for notifications
const successMessage = ref<string | null>(null);
const error = ref<string | null>(null);

// Active tab for settings
const activeTab = ref('profile');

// Settings tabs
const settingsTabs = [
  { 
    id: 'profile', 
    label: 'Profile', 
    icon: 'user',
    color: '#FF66B3'
  },
  { 
    id: 'appearance', 
    label: 'Appearance', 
    icon: 'palette',
    color: '#8B5CF6'
  },
  { 
    id: 'notifications', 
    label: 'Notifications', 
    icon: 'bell',
    color: '#FDBD10'
  },
  { 
    id: 'security', 
    label: 'Security', 
    icon: 'shield',
    color: '#00C951'
  }
];

// Clear messages after a delay
const clearMessages = () => {
  setTimeout(() => {
  successMessage.value = null;
  error.value = null;
  }, 5000);
};

// Handle success messages
const handleSuccess = (message: string) => {
  successMessage.value = message;
  error.value = null;
  clearMessages();
};

// Handle error messages  
const handleError = (message: string) => {
  error.value = message;
  successMessage.value = null;
  clearMessages();
};

// Tab icon renderer
const renderTabIcon = (iconName: string) => {
  const icons = {
    user: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />',
    palette: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.098 19.902a3.75 3.75 0 005.304 0l6.401-6.402M6.75 21A3.75 3.75 0 013 17.25V4.125C3 3.504 3.504 3 4.125 3h5.25c.621 0 1.125.504 1.125 1.125v4.072M6.75 21a3.75 3.75 0 003.75-3.75V8.197M6.75 21h13.125c.621 0 1.125-.504 1.125-1.125v-5.25c0-.621-.504-1.125-1.125-1.125h-4.072M10.5 8.197l2.88-2.88c.438-.439 1.15-.439 1.59 0l3.712 3.713c.44.44.44 1.152 0 1.59l-2.879 2.88M6.75 17.25h.008v.008H6.75v-.008z" />',
    bell: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />',
    shield: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />'
  };
  return icons[iconName as keyof typeof icons] || '';
};
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div class="pt-4 px-6 flex justify-between items-center">
      <BackButton fallbackRoute="/" label="Back to Dashboard" />
    </div>
    
    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
      <!-- Page Header -->
      <div class="mb-6">
        <h1 class="text-2xl font-bold text-white">Settings</h1>
        <p class="text-slate-400 mt-2">
          Manage your profile, preferences, and security settings
        </p>
      </div>

      <!-- Success/Error messages -->
      <div v-if="successMessage" class="p-4 bg-green-900/50 text-green-400 rounded-lg">
        {{ successMessage }}
      </div>
      <div v-if="error" class="p-4 bg-red-900/50 text-red-400 rounded-lg">
        {{ error }}
      </div>

      <!-- Main content -->
      <div class="flex gap-6">
        <!-- Sidebar Navigation -->
        <div class="w-64 flex-shrink-0">
          <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
            <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
              <h2 class="text-lg font-medium text-white">Settings</h2>
            </div>
            <nav class="p-2 flex flex-col gap-1">
            <button
              v-for="tab in settingsTabs"
              :key="tab.id"
              @click="activeTab = tab.id"
                class="rounded-lg transition-colors duration-200 text-white flex items-center gap-3 relative overflow-hidden px-3 py-2"
              :class="[
                activeTab === tab.id
                    ? 'bg-slate-700/50 border border-slate-600/30 text-white font-medium'
                    : 'text-slate-300 hover:bg-slate-700/30 hover:text-white border border-transparent'
              ]"
            >
              <!-- Active indicator bar -->
              <div
                v-if="activeTab === tab.id"
                  class="absolute left-0 top-0 bottom-0 w-1 rounded-r"
                :style="{ backgroundColor: tab.color }"
              ></div>
              
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" v-html="renderTabIcon(tab.icon)"></svg>
              <span class="text-sm whitespace-nowrap">{{ tab.label }}</span>
            </button>
          </nav>
          </div>
        </div>

        <!-- Content Area -->
        <div class="flex-1">
          <!-- Profile Tab -->
          <div v-if="activeTab === 'profile'" class="flex flex-col gap-6">
            <UserProfileCard
              :can-edit="true"
              :show-editable-fields="true"
              @success="handleSuccess"
              @error="handleError"
            />
          </div>

          <!-- Appearance Tab -->
          <div v-if="activeTab === 'appearance'">
            <AppearanceSettings
              @success="handleSuccess"
              @error="handleError"
            />
          </div>

          <!-- Notifications Tab -->
          <div v-if="activeTab === 'notifications'">
            <NotificationSettings
              @success="handleSuccess"
              @error="handleError"
            />
          </div>

          <!-- Security Tab -->
          <div v-if="activeTab === 'security'" class="flex flex-col gap-4">
            <SecuritySettings
              @success="handleSuccess"
              @error="handleError"
            />
            <MFASettings
              @success="handleSuccess"
              @error="handleError"
            />
            <AuthMethodsSettings
              @success="handleSuccess"
              @error="handleError"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Custom toggle switch styling */
.peer:checked ~ div {
  background-color: rgb(37 99 235);
}

.peer:checked ~ div:after {
  transform: translateX(100%);
  border-color: white;
}

/* Smooth transitions for theme selection */
.theme-option {
  transition: all 0.2s ease-in-out;
}

.theme-option:hover {
  transform: translateY(-1px);
}

/* Custom scrollbar for long content */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: rgb(51 65 85);
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgb(100 116 139);
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgb(148 163 184);
}
</style> 