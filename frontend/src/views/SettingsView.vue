<script setup lang="ts">
import { ref } from 'vue';
import SettingsCard from '@/components/settings/SettingsCard.vue';
import SettingsToggle from '@/components/settings/SettingsToggle.vue';

const activeTab = ref('appearance');

// Add refs for toggle states
const darkMode = ref(true); // Default to dark mode enabled
const compactView = ref(false);
const emailNotifications = ref(true);
const desktopNotifications = ref(false);
const twoFactorAuth = ref(false);

// Theme selection
const selectedTheme = ref('slate-dark');
const themes = [
  { id: 'slate-dark', name: 'Slate Dark', description: 'Default dark blue theme' },
  { id: 'midnight', name: 'Midnight', description: 'Deep blue/black theme' },
  { id: 'nord', name: 'Nord', description: 'Cool blue arctic theme' },
  { id: 'monokai', name: 'Monokai', description: 'Vibrant dark theme with colored accents' },
  { id: 'solarized-dark', name: 'Solarized Dark', description: 'Ethan Schoonover\'s dark theme' },
  { id: 'dracula', name: 'Dracula', description: 'Dark theme with purple accents' },
  { id: 'github-dark', name: 'GitHub Dark', description: 'GitHub\'s dark theme' },
  { id: 'light', name: 'Light Mode', description: 'Light theme for daytime use' },
];

interface SettingsTab {
  id: string;
  label: string;
}

const tabs: SettingsTab[] = [
  { id: 'appearance', label: 'Appearance' },
  { id: 'notifications', label: 'Notifications' },
  { id: 'account', label: 'Account' },
  { id: 'security', label: 'Security' }
];

const applyTheme = (themeId: string) => {
  selectedTheme.value = themeId;
  // TODO: Implement actual theme application logic
  console.log(`Theme changed to: ${themeId}`);
};
</script>

<template>
  <div class="flex-1 bg-slate-900">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="flex flex-col gap-8">
        <!-- Header -->
        <div>
          <h1 class="text-2xl font-semibold text-white">Settings</h1>
          <p class="text-slate-400 mt-1">Manage your account settings and preferences</p>
        </div>

        <!-- Settings Grid -->
        <div class="flex gap-8">
          <!-- Sidebar -->
          <div class="w-64">
            <nav class="flex flex-col gap-1">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                @click="activeTab = tab.id"
                :class="[
                  activeTab === tab.id
                    ? 'bg-slate-800 text-white'
                    : 'text-slate-400 hover:text-white hover:bg-slate-800',
                  'group flex items-center px-3 py-2 text-sm font-medium rounded-md w-full'
                ]"
              >
                {{ tab.label }}
              </button>
            </nav>
          </div>

          <!-- Content -->
          <div class="flex-1">
            <!-- Appearance -->
            <div v-if="activeTab === 'appearance'" class="flex flex-col gap-6">
              <SettingsCard title="Appearance Settings">
                <SettingsToggle
                  title="Dark Mode"
                  description="Enable dark mode throughout the application"
                  v-model="darkMode"
                />
                
                <SettingsToggle
                  title="Compact View"
                  description="Show more content with reduced spacing"
                  v-model="compactView"
                />
              </SettingsCard>
              
              <!-- Theme Selection Card -->
              <SettingsCard title="Theme Selection">
                <div class="flex flex-col gap-4">
                  <p class="text-sm text-slate-400">Choose a theme for the application interface</p>
                  
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div 
                      v-for="theme in themes" 
                      :key="theme.id"
                      @click="applyTheme(theme.id)"
                      class="relative flex cursor-pointer rounded-lg border p-4 focus:outline-none"
                      :class="selectedTheme === theme.id ? 'bg-slate-700 border-blue-500 ring-2 ring-blue-500' : 'bg-slate-800 border-slate-700 hover:bg-slate-750'"
                    >
                      <span class="flex flex-1">
                        <span class="flex flex-col">
                          <span class="block text-sm font-medium text-white">{{ theme.name }}</span>
                          <span class="mt-1 flex items-center text-xs text-slate-400">{{ theme.description }}</span>
                        </span>
                      </span>
                      <span 
                        class="pointer-events-none absolute -inset-px rounded-lg" 
                        aria-hidden="true" 
                      ></span>
                    </div>
                  </div>
                  
                  <!-- Theme Preview Section -->
                  <div class="mt-6 p-4 bg-slate-800 rounded-lg border border-slate-700">
                    <h4 class="text-sm font-medium text-white mb-2">Preview</h4>
                    <div class="h-32 rounded-md border border-slate-700 flex items-center justify-center">
                      <p class="text-sm text-slate-400">Theme preview coming soon</p>
                    </div>
                  </div>
                </div>
              </SettingsCard>
            </div>

            <!-- Notifications -->
            <div v-if="activeTab === 'notifications'" class="flex flex-col gap-6">
              <SettingsCard title="Notification Preferences">
                <SettingsToggle
                  title="Email Notifications"
                  description="Receive email notifications for important updates"
                  v-model="emailNotifications"
                />
                
                <SettingsToggle
                  title="Desktop Notifications"
                  description="Show desktop notifications when you're using the app"
                  v-model="desktopNotifications"
                />
              </SettingsCard>
            </div>

            <!-- Account -->
            <div v-if="activeTab === 'account'" class="flex flex-col gap-6">
              <SettingsCard title="Account Settings">
                <div class="bg-slate-700/50 rounded-md p-4">
                  <div class="mb-4">
                    <label class="block text-sm font-medium text-white mb-2">Name</label>
                    <input type="text" class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md shadow-sm text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-white mb-2">Email</label>
                    <input type="email" class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md shadow-sm text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
                  </div>
                </div>
                
                <div class="flex justify-end">
                  <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
                    Save Changes
                  </button>
                </div>
              </SettingsCard>
            </div>

            <!-- Security -->
            <div v-if="activeTab === 'security'" class="flex flex-col gap-6">
              <SettingsCard title="Security Settings">
                <div class="bg-slate-700/50 rounded-md p-4">
                  <h3 class="text-sm font-medium text-white mb-3">Password</h3>
                  <p class="text-sm text-slate-400 mb-4">Change your password or reset it if you've forgotten it</p>
                  <button class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 transition-colors">
                    Change Password
                  </button>
                </div>
                
                <SettingsToggle
                  title="Two-factor Authentication"
                  description="Add an extra layer of security to your account"
                  v-model="twoFactorAuth"
                />
              </SettingsCard>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
