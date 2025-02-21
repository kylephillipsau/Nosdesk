<script setup lang="ts">
import { ref } from 'vue';

const activeTab = ref('appearance');

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
            <nav class="space-y-1">
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
          <div class="flex-1 bg-slate-800 rounded-lg p-6">
            <!-- Appearance -->
            <div v-if="activeTab === 'appearance'" class="space-y-6">
              <h2 class="text-lg font-medium text-white">Appearance Settings</h2>
              <div class="space-y-4">
                <div>
                  <label class="flex items-center">
                    <input type="checkbox" class="form-checkbox bg-slate-700 border-slate-600 rounded text-blue-500" />
                    <span class="ml-2 text-slate-300">Enable dark mode</span>
                  </label>
                </div>
                <div>
                  <label class="flex items-center">
                    <input type="checkbox" class="form-checkbox bg-slate-700 border-slate-600 rounded text-blue-500" />
                    <span class="ml-2 text-slate-300">Show compact view</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Notifications -->
            <div v-if="activeTab === 'notifications'" class="space-y-6">
              <h2 class="text-lg font-medium text-white">Notification Preferences</h2>
              <div class="space-y-4">
                <div>
                  <label class="flex items-center">
                    <input type="checkbox" class="form-checkbox bg-slate-700 border-slate-600 rounded text-blue-500" />
                    <span class="ml-2 text-slate-300">Email notifications</span>
                  </label>
                </div>
                <div>
                  <label class="flex items-center">
                    <input type="checkbox" class="form-checkbox bg-slate-700 border-slate-600 rounded text-blue-500" />
                    <span class="ml-2 text-slate-300">Desktop notifications</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Account -->
            <div v-if="activeTab === 'account'" class="space-y-6">
              <h2 class="text-lg font-medium text-white">Account Settings</h2>
              <div class="space-y-4">
                <div>
                  <label class="block text-sm font-medium text-slate-300">Name</label>
                  <input type="text" class="mt-1 block w-full bg-slate-700 border border-slate-600 rounded-md shadow-sm text-white" />
                </div>
                <div>
                  <label class="block text-sm font-medium text-slate-300">Email</label>
                  <input type="email" class="mt-1 block w-full bg-slate-700 border border-slate-600 rounded-md shadow-sm text-white" />
                </div>
              </div>
            </div>

            <!-- Security -->
            <div v-if="activeTab === 'security'" class="space-y-6">
              <h2 class="text-lg font-medium text-white">Security Settings</h2>
              <div class="space-y-4">
                <button class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600">
                  Change Password
                </button>
                <div>
                  <label class="flex items-center">
                    <input type="checkbox" class="form-checkbox bg-slate-700 border-slate-600 rounded text-blue-500" />
                    <span class="ml-2 text-slate-300">Enable two-factor authentication</span>
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
