<script setup lang="ts">
import { ref } from 'vue';
import ToggleSwitch from '@/components/common/ToggleSwitch.vue';

// Notification preferences
const emailNotifications = ref(true);
const desktopNotifications = ref(false);

// Emits for notifications
const emit = defineEmits<{
  (e: 'success', message: string): void;
  (e: 'error', message: string): void;
}>();

// Handle notification toggles
const handleEmailNotificationToggle = () => {
  // TODO: Implement save to backend/store
  emit('success', `Email notifications ${emailNotifications.value ? 'enabled' : 'disabled'}`);
};

const handleDesktopNotificationToggle = () => {
  // TODO: Implement save to backend/store and request browser permission if enabling
  if (desktopNotifications.value && 'Notification' in window) {
    Notification.requestPermission().then((permission) => {
      if (permission === 'granted') {
        emit('success', 'Desktop notifications enabled');
      } else {
        desktopNotifications.value = false;
        emit('error', 'Desktop notification permission denied');
      }
    });
  } else {
    emit('success', `Desktop notifications ${desktopNotifications.value ? 'enabled' : 'disabled'}`);
  }
};
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <h2 class="text-lg font-medium text-white">Notifications</h2>
      <p class="text-sm text-slate-400 mt-1">Configure how you'd like to be notified about updates</p>
    </div>
    
    <div class="p-6 flex flex-col gap-3">
      <!-- Email Notifications -->
      <ToggleSwitch
        v-model="emailNotifications"
        label="Email notifications"
        description="Get notified about important updates via email"
        @update:modelValue="handleEmailNotificationToggle"
      />

      <!-- Desktop Notifications -->
      <ToggleSwitch
        v-model="desktopNotifications"
        label="Desktop notifications"
        description="Show notifications on your desktop when the app is closed"
        @update:modelValue="handleDesktopNotificationToggle"
      />
    </div>
  </div>
</template> 