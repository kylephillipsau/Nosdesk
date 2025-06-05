<script setup lang="ts">
import { ref, onMounted } from 'vue';

interface AuthMethod {
  id: string;
  type: 'email' | 'google' | 'github';
  identifier: string;
  isPrimary: boolean;
  createdAt: string;
}

interface ActiveSession {
  id: string;
  device: string;
  location: string;
  lastActive: string;
  isCurrent: boolean;
}

// State
const authMethods = ref<AuthMethod[]>([]);
const activeSessions = ref<ActiveSession[]>([]);
const loading = ref(false);

// Emits for notifications
const emit = defineEmits<{
  (e: 'success', message: string): void;
  (e: 'error', message: string): void;
}>();

// Load data on mount
onMounted(async () => {
  await loadAuthMethods();
  await loadActiveSessions();
});

// Mock data loading functions
const loadAuthMethods = async () => {
  // TODO: Replace with actual API call
  authMethods.value = [
    {
      id: '1',
      type: 'email',
      identifier: 'user@example.com',
      isPrimary: true,
      createdAt: '2024-01-01'
    }
  ];
};

const loadActiveSessions = async () => {
  // TODO: Replace with actual API call
  activeSessions.value = [
    {
      id: '1',
      device: 'MacBook Pro - Chrome',
      location: 'San Francisco, CA',
      lastActive: '2024-01-15T10:30:00Z',
      isCurrent: true
    },
    {
      id: '2',
      device: 'iPhone - Safari',
      location: 'San Francisco, CA',
      lastActive: '2024-01-14T15:20:00Z',
      isCurrent: false
    }
  ];
};

// Auth method functions
const addAuthMethod = async (type: 'google' | 'github') => {
  loading.value = true;
  try {
    // TODO: Implement OAuth flow for adding auth method
    emit('success', `${type.charAt(0).toUpperCase() + type.slice(1)} account linked successfully`);
  } catch (err) {
    emit('error', `Failed to link ${type} account`);
    console.error(`Error linking ${type} account:`, err);
  } finally {
    loading.value = false;
  }
};

const removeAuthMethod = async (methodId: string) => {
  if (authMethods.value.length <= 1) {
    emit('error', 'You must have at least one authentication method');
    return;
  }

  loading.value = true;
  try {
    // TODO: Implement auth method removal API call
    authMethods.value = authMethods.value.filter(method => method.id !== methodId);
    emit('success', 'Authentication method removed successfully');
  } catch (err) {
    emit('error', 'Failed to remove authentication method');
    console.error('Error removing auth method:', err);
  } finally {
    loading.value = false;
  }
};

// Session functions
const revokeSession = async (sessionId: string) => {
  loading.value = true;
  try {
    // TODO: Implement session revocation API call
    activeSessions.value = activeSessions.value.filter(session => session.id !== sessionId);
    emit('success', 'Session revoked successfully');
  } catch (err) {
    emit('error', 'Failed to revoke session');
    console.error('Error revoking session:', err);
  } finally {
    loading.value = false;
  }
};

const revokeAllSessions = async () => {
  loading.value = true;
  try {
    // TODO: Implement revoke all sessions API call
    activeSessions.value = activeSessions.value.filter(session => session.isCurrent);
    emit('success', 'All other sessions revoked successfully');
  } catch (err) {
    emit('error', 'Failed to revoke sessions');
    console.error('Error revoking all sessions:', err);
  } finally {
    loading.value = false;
  }
};

// Utility functions
const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
};

const getAuthMethodIcon = (type: string) => {
  switch (type) {
    case 'google':
      return '🔍';
    case 'github':
      return '🐙';
    default:
      return '📧';
  }
};
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Authentication Methods -->
    <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
      <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
        <h2 class="text-lg font-medium text-white">Authentication Methods</h2>
        <p class="text-sm text-slate-400 mt-1">Manage how you sign in to your account</p>
      </div>
      
      <div class="p-6 flex flex-col gap-4">
        <!-- Existing Auth Methods -->
        <div class="flex flex-col gap-3">
          <h3 class="text-sm font-medium text-white">Connected Accounts</h3>
          <div class="flex flex-col gap-2">
            <div v-for="method in authMethods" :key="method.id" class="flex items-center justify-between p-3 bg-slate-700/30 rounded-lg">
              <div class="flex items-center gap-3">
                <span class="text-xl">{{ getAuthMethodIcon(method.type) }}</span>
                <div>
                  <div class="text-sm font-medium text-white">
                    {{ method.identifier }}
                    <span v-if="method.isPrimary" class="ml-2 px-2 py-1 bg-blue-600/20 text-blue-400 rounded text-xs">Primary</span>
                  </div>
                  <div class="text-xs text-slate-400">
                    Added {{ formatDate(method.createdAt) }}
                  </div>
                </div>
              </div>
              <button
                v-if="!method.isPrimary && authMethods.length > 1"
                @click="removeAuthMethod(method.id)"
                :disabled="loading"
                class="text-red-400 hover:text-red-300 text-sm font-medium disabled:opacity-50"
              >
                Remove
              </button>
            </div>
          </div>
        </div>

        <!-- Add Auth Methods -->
        <div class="flex flex-col gap-3">
          <h3 class="text-sm font-medium text-white">Add Authentication Method</h3>
          <div class="grid grid-cols-2 gap-3">
            <button
              @click="addAuthMethod('google')"
              :disabled="loading"
              class="flex items-center gap-3 p-3 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg border border-slate-600/30 hover:border-slate-500/50 transition-colors disabled:opacity-50"
            >
              <span class="text-xl">🔍</span>
              <span class="text-sm font-medium text-white">Google</span>
            </button>
            <button
              @click="addAuthMethod('github')"
              :disabled="loading"
              class="flex items-center gap-3 p-3 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg border border-slate-600/30 hover:border-slate-500/50 transition-colors disabled:opacity-50"
            >
              <span class="text-xl">🐙</span>
              <span class="text-sm font-medium text-white">GitHub</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Active Sessions -->
    <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
      <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
        <h2 class="text-lg font-medium text-white">Active Sessions</h2>
        <p class="text-sm text-slate-400 mt-1">Manage your active login sessions</p>
      </div>
      
      <div class="p-6 flex flex-col gap-4">
        <!-- Sessions List -->
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-medium text-white">Current Sessions</h3>
            <button
              @click="revokeAllSessions"
              :disabled="loading || activeSessions.length <= 1"
              class="text-red-400 hover:text-red-300 text-sm font-medium disabled:opacity-50"
            >
              Revoke All Others
            </button>
          </div>
          
          <div class="flex flex-col gap-2">
            <div v-for="session in activeSessions" :key="session.id" class="flex items-center justify-between p-3 bg-slate-700/30 rounded-lg">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 bg-slate-600 rounded-lg flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-slate-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                  </svg>
                </div>
                <div>
                  <div class="text-sm font-medium text-white">
                    {{ session.device }}
                    <span v-if="session.isCurrent" class="ml-2 px-2 py-1 bg-green-600/20 text-green-400 rounded text-xs">Current</span>
                  </div>
                  <div class="text-xs text-slate-400">
                    {{ session.location }} • Last active {{ formatDate(session.lastActive) }}
                  </div>
                </div>
              </div>
              <button
                v-if="!session.isCurrent"
                @click="revokeSession(session.id)"
                :disabled="loading"
                class="text-red-400 hover:text-red-300 text-sm font-medium disabled:opacity-50"
              >
                Revoke
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template> 