<!-- // Microsoft Configuration View -->
<template>
  <div class="p-4">
    <div class="max-w-2xl mx-auto">
      <div class="mb-6">
        <label for="clientId" class="text-sm font-medium text-white mb-2 block">Client ID</label>
        <input
          id="clientId"
          v-model="config.clientId"
          type="text"
          class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
          placeholder="Enter Client ID"
        />
      </div>

      <div class="mb-6">
        <label for="clientSecret" class="text-sm font-medium text-white mb-2 block">Client Secret</label>
        <input
          id="clientSecret"
          v-model="config.clientSecret"
          type="password"
          class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
          placeholder="Enter Client Secret"
        />
      </div>

      <div class="mb-6">
        <label for="tenantId" class="text-sm font-medium text-white mb-2 block">Tenant ID</label>
        <input
          id="tenantId"
          v-model="config.tenantId"
          type="text"
          class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
          placeholder="Enter Tenant ID"
        />
      </div>

      <div class="mb-6">
        <label for="redirectUri" class="text-sm font-medium text-white mb-2 block">Redirect URI</label>
        <input
          id="redirectUri"
          v-model="config.redirectUri"
          type="text"
          class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
          placeholder="Enter Redirect URI"
        />
      </div>

      <div class="flex gap-3 mt-8">
        <button
          @click="saveConfiguration"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          :disabled="isLoading"
        >
          <span v-if="isLoading" class="animate-spin h-4 w-4">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </span>
          {{ isLoading ? 'Saving...' : 'Save Configuration' }}
        </button>
        
        <button
          @click="testConfiguration"
          class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          :disabled="isLoading"
        >
          <span v-if="isLoading" class="animate-spin h-4 w-4">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </span>
          {{ isLoading ? 'Testing...' : 'Test Configuration' }}
        </button>
      </div>

      <!-- Success Message -->
      <div v-if="successMessage" class="mt-4 p-4 bg-green-900/50 text-green-400 rounded-lg border border-green-700">
        <pre class="whitespace-pre-wrap text-sm">{{ successMessage }}</pre>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="mt-4 p-4 bg-red-900/50 text-red-400 rounded-lg border border-red-700">
        <pre class="whitespace-pre-wrap text-sm">{{ errorMessage }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import axios from 'axios';

const props = defineProps<{
  providerId?: number;
}>();

const config = ref({
  clientId: '',
  clientSecret: '',
  tenantId: '',
  redirectUri: ''
});

const isLoading = ref(false);
const errorMessage = ref<string | null>(null);
const successMessage = ref<string | null>(null);

// Load existing configuration
const loadConfiguration = async () => {
  if (!props.providerId) return;
  
  try {
    const response = await axios.get(`/api/auth/providers/${props.providerId}`);
    if (response.data.config) {
      config.value = {
        clientId: response.data.config.client_id || '',
        clientSecret: response.data.config.client_secret || '',
        tenantId: response.data.config.tenant_id || '',
        redirectUri: response.data.config.redirect_uri || ''
      };
    }
  } catch (error: any) {
    console.error('Error loading configuration:', error);
    errorMessage.value = 'Failed to load configuration';
  }
};

// Save configuration
const saveConfiguration = async () => {
  if (!props.providerId) {
    errorMessage.value = 'No provider ID available';
    return;
  }

  isLoading.value = true;
  errorMessage.value = null;
  successMessage.value = null;

  try {
    await axios.put(`/api/auth/providers/${props.providerId}`, {
      config: {
        client_id: config.value.clientId,
        client_secret: config.value.clientSecret,
        tenant_id: config.value.tenantId,
        redirect_uri: config.value.redirectUri
      }
    });
    
    successMessage.value = 'Configuration saved successfully';
  } catch (error: any) {
    console.error('Error saving configuration:', error);
    errorMessage.value = error.response?.data?.message || 'Failed to save configuration';
  } finally {
    isLoading.value = false;
  }
};

// Test configuration
const testConfiguration = async () => {
  if (!props.providerId) {
    errorMessage.value = 'No provider ID available for testing';
    return;
  }

  isLoading.value = true;
  errorMessage.value = null;
  successMessage.value = null;
  
  try {
    const response = await axios.get(`/api/auth/providers/${props.providerId}/test`);
    
    if (response.data.status === 'success') {
      successMessage.value = response.data.message;
      
      if (response.data.details) {
        const details = response.data.details;
        successMessage.value += '\n\nConfiguration status:';
        if (details.client_id_valid) successMessage.value += '\n✓ Client ID is valid';
        if (details.tenant_id_valid) successMessage.value += '\n✓ Tenant ID is valid';
        if (details.client_secret_valid) successMessage.value += '\n✓ Client Secret is valid';
        if (details.redirect_uri_configured) successMessage.value += '\n✓ Redirect URI is configured';
      }
    } else {
      errorMessage.value = response.data.message;
      
      if (response.data.details) {
        errorMessage.value += `\n\nDetails: ${response.data.details}`;
      }
    }
  } catch (error: any) {
    console.error('Error testing Microsoft configuration:', error);
    errorMessage.value = error.response?.data?.message || 'Failed to test configuration';
  } finally {
    isLoading.value = false;
  }
};

// Load configuration on mount
onMounted(() => {
  loadConfiguration();
});
</script>