<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';

// Define types for our data structures
interface ProviderConfig {
  key: string;
  value: string;
  is_secret: boolean;
}

interface Provider {
  id: string;
  name: string;
  provider_type: string;
  enabled: boolean;
  is_default: boolean;
  configs?: ProviderConfig[];
}

const router = useRouter();

// State for providers
const isLoading = ref(false);
const errorMessage = ref('');
const providers = ref<Provider[]>([]);
const successMessage = ref('');

// Selected provider for configuration
const selectedProvider = ref<Provider | null>(null);
const showConfigModal = ref(false);
const showAddProviderModal = ref(false);
const newProviderType = ref('microsoft');

// Client ID and Tenant ID for Microsoft Entra
const clientId = ref('');
const tenantId = ref('');
const clientSecret = ref('');
const redirectUri = ref('');

// Load providers from API
const loadProviders = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    const response = await axios.get('/api/auth/providers');
    providers.value = response.data;
  } catch (error: any) {
    console.error('Failed to load auth providers:', error);
    errorMessage.value = error.response?.data?.message || 'Failed to load authentication providers';
  } finally {
    isLoading.value = false;
  }
};

// Toggle provider enabled state
const toggleProvider = async (provider: Provider) => {
  const updatedProvider = { 
    name: provider.name,
    enabled: !provider.enabled,
    is_default: provider.is_default
  };
  
  try {
    await axios.put(`/api/auth/providers/${provider.id}`, updatedProvider);
    
    // Reload providers to get updated data
    await loadProviders();
    
    successMessage.value = `${provider.name} ${updatedProvider.enabled ? 'enabled' : 'disabled'} successfully`;
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } catch (error: any) {
    console.error(`Failed to update ${provider.name}:`, error);
    errorMessage.value = error.response?.data?.message || `Failed to update ${provider.name}`;
    setTimeout(() => { errorMessage.value = ''; }, 3000);
  }
};

// Create a new provider
const createProvider = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    const newProvider = {
      name: newProviderType.value === 'microsoft' ? 'Microsoft Entra' : 'New Provider',
      provider_type: newProviderType.value,
      enabled: false,
      is_default: false
    };
    
    const response = await axios.post('/api/auth/providers', newProvider);
    
    // Close the modal and reload providers
    showAddProviderModal.value = false;
    await loadProviders();
    
    // Open configuration modal for the new provider
    const createdProvider = response.data;
    configureProvider(createdProvider);
    
    successMessage.value = `${newProvider.name} provider created successfully`;
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } catch (error: any) {
    console.error('Failed to create provider:', error);
    errorMessage.value = error.response?.data?.message || 'Failed to create provider';
  } finally {
    isLoading.value = false;
  }
};

// Open configuration modal for a provider
const configureProvider = (provider: Provider) => {
  selectedProvider.value = provider;
  
  // Initialize form values from provider config
  if (provider.provider_type === 'microsoft') {
    const configs = provider.configs || [];
    
    clientId.value = configs.find((c) => c.key === 'client_id')?.value || '';
    tenantId.value = configs.find((c) => c.key === 'tenant_id')?.value || '';
    clientSecret.value = ''; // We don't store or display secrets
    redirectUri.value = configs.find((c) => c.key === 'redirect_uri')?.value || 
      `${window.location.origin}/auth/microsoft/callback`;
  }
  
  showConfigModal.value = true;
};

// Save provider configuration
const saveProviderConfig = async () => {
  if (!selectedProvider.value) return;
  
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    // Create config request based on provider type
    const configRequest: {
      provider_id: string;
      configs: ProviderConfig[];
    } = {
      provider_id: selectedProvider.value.id,
      configs: []
    };
    
    if (selectedProvider.value.provider_type === 'microsoft') {
      configRequest.configs = [
        { key: 'client_id', value: clientId.value, is_secret: false },
        { key: 'tenant_id', value: tenantId.value, is_secret: false },
        { key: 'redirect_uri', value: redirectUri.value, is_secret: false }
      ];
      
      // Only include client secret if it's provided
      if (clientSecret.value.trim()) {
        configRequest.configs.push({
          key: 'client_secret',
          value: clientSecret.value,
          is_secret: true
        });
      }
    }
    
    // Send configuration to backend
    await axios.post(`/api/auth/providers/config`, configRequest);
    
    // Close the modal and reload providers
    showConfigModal.value = false;
    await loadProviders();
    
    successMessage.value = `${selectedProvider.value.name} configuration updated successfully`;
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } catch (error: any) {
    console.error(`Failed to save ${selectedProvider.value.name} configuration:`, error);
    errorMessage.value = error.response?.data?.message || `Failed to save ${selectedProvider.value.name} configuration`;
  } finally {
    isLoading.value = false;
  }
};

// Set a provider as default
const setAsDefault = async (provider: Provider) => {
  try {
    await axios.post(`/api/auth/providers/default`, { provider_id: provider.id });
    
    // Update the UI
    await loadProviders();
    
    successMessage.value = `${provider.name} set as default authentication method`;
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } catch (error: any) {
    console.error(`Failed to set ${provider.name} as default:`, error);
    errorMessage.value = error.response?.data?.message || `Failed to set ${provider.name} as default`;
    setTimeout(() => { errorMessage.value = ''; }, 3000);
  }
};

// Navigate back to admin settings
const goBack = () => {
  router.push('/admin/settings');
};

// Helper function to render SVG icons
const getProviderIcon = (iconName: string) => {
  switch (iconName) {
    case 'microsoft':
      return `
        <svg width="16" height="16" viewBox="0 0 21 21" class="mr-2">
          <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
          <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
          <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
          <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
        </svg>
      `;
    case 'google':
      return `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
          <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.1 5.1 0 0 1-2.2 3.35v2.74h3.56c2.08-1.92 3.28-4.74 3.28-8.1z"/>
          <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77a6.45 6.45 0 0 1-3.71 1.07 6.41 6.41 0 0 1-6-4.18H2.4v2.82A11.46 11.46 0 0 0 12 23z"/>
          <path fill="#FBBC05" d="M5.99 14.54a6.6 6.6 0 0 1-.38-2.23c0-.77.14-1.52.38-2.22V7.27H2.4a11.54 11.54 0 0 0 0 10.46l3.59-3.19z"/>
          <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.2 1.64l3.15-3.15A10.56 10.56 0 0 0 12 1C7.33 1 3.28 3.83 1.32 7.75l3.66 2.84A6.53 6.53 0 0 1 12 5.38z"/>
        </svg>
      `;
    case 'user':
      return `
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
      `;
    case 'key':
      return `
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
        </svg>
      `;
    default:
      return `
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      `;
  }
};

// Function to test auth provider
const testAuthProvider = async (provider: Provider) => {
  if (provider.provider_type === 'microsoft') {
    try {
      // Store the current URL to redirect back after authentication
      sessionStorage.setItem('authRedirect', router.currentRoute.value.fullPath);
      
      // Get authorization URL from backend
      const response = await axios.post('/api/auth/oauth/authorize', {
        provider_type: 'microsoft',
        redirect_uri: `${window.location.origin}/auth/microsoft/callback` 
      });
      
      // Redirect to Microsoft login
      window.location.href = response.data.auth_url;
    } catch (error: any) {
      console.error('Error initiating Microsoft authentication:', error);
      errorMessage.value = error.response?.data?.message || 'Failed to initiate Microsoft authentication';
      setTimeout(() => { errorMessage.value = ''; }, 3000);
    }
  }
};

onMounted(() => {
  loadProviders();
});
</script>

<template>
  <div class="auth-providers-view p-4">
    <div class="flex justify-between items-center mb-6">
      <h1 class="text-2xl font-bold text-white">Authentication Providers</h1>
      <button 
        @click="goBack" 
        class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 transition-colors"
      >
        Back to Settings
      </button>
    </div>
    
    <!-- Success message -->
    <div 
      v-if="successMessage" 
      class="mb-4 p-3 bg-green-900/50 border border-green-700 text-green-200 rounded-md flex items-center"
    >
      <span class="mr-2">✅</span>
      {{ successMessage }}
    </div>
    
    <!-- Error message -->
    <div 
      v-if="errorMessage" 
      class="mb-4 p-3 bg-red-900/50 border border-red-700 text-red-200 rounded-md flex items-center"
    >
      <span class="mr-2">❌</span>
      {{ errorMessage }}
    </div>
    
    <!-- Add Provider Button -->
    <div class="mb-4">
      <button 
        @click="showAddProviderModal = true"
        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors flex items-center gap-2"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add Authentication Provider
      </button>
    </div>
    
    <!-- Loading state -->
    <div v-if="isLoading" class="flex justify-center my-8">
      <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
    </div>
    
    <!-- Provider list -->
    <div v-else class="space-y-4">
      <div v-for="provider in providers" :key="provider.id" 
           class="bg-slate-800 p-4 rounded-lg shadow border border-slate-700 flex flex-col md:flex-row md:items-center md:justify-between">
        
        <div class="flex items-center mb-3 md:mb-0">
          <!-- Provider icon -->
          <div class="mr-3 text-slate-300" v-html="getProviderIcon(provider.provider_type)"></div>
          
          <!-- Provider name and status -->
          <div>
            <div class="flex items-center">
              <span class="font-medium text-lg text-white">{{ provider.name }}</span>
              <span v-if="provider.is_default" 
                    class="ml-2 px-2 py-0.5 text-xs bg-blue-900/50 text-blue-200 rounded-full border border-blue-700">
                Default
              </span>
            </div>
            <div class="text-sm text-slate-400">
              {{ provider.enabled ? 'Enabled' : 'Disabled' }}
            </div>
          </div>
        </div>
        
        <!-- Provider actions -->
        <div class="flex flex-wrap gap-2">
          <button 
            @click="toggleProvider(provider)"
            class="px-3 py-1 rounded-md text-sm"
            :class="provider.enabled ? 'bg-red-900/50 text-red-200 hover:bg-red-800/50 border border-red-700' : 'bg-green-900/50 text-green-200 hover:bg-green-800/50 border border-green-700'"
          >
            {{ provider.enabled ? 'Disable' : 'Enable' }}
          </button>
          
          <button 
            @click="configureProvider(provider)"
            class="px-3 py-1 bg-slate-700 text-slate-200 rounded-md text-sm hover:bg-slate-600 border border-slate-600"
          >
            Configure
          </button>
          
          <button 
            v-if="provider.enabled && !provider.is_default"
            @click="setAsDefault(provider)"
            class="px-3 py-1 bg-blue-900/50 text-blue-200 rounded-md text-sm hover:bg-blue-800/50 border border-blue-700"
          >
            Set as Default
          </button>
          
          <button 
            v-if="provider.enabled"
            @click="testAuthProvider(provider)"
            class="px-3 py-1 bg-purple-900/50 text-purple-200 rounded-md text-sm hover:bg-purple-800/50 border border-purple-700"
          >
            Test
          </button>
        </div>
      </div>
      
      <div v-if="providers.length === 0 && !isLoading" class="text-center py-8 text-slate-400">
        No authentication providers found
      </div>
    </div>
    
    <!-- Add Provider Modal -->
    <div v-if="showAddProviderModal" class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center p-4 z-50">
      <div class="bg-slate-800 rounded-lg shadow-lg p-6 w-full max-w-lg border border-slate-700">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-semibold text-white">
            Add Authentication Provider
          </h2>
          <button @click="showAddProviderModal = false" class="text-slate-400 hover:text-white">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-1">
              Provider Type
            </label>
            <select 
              v-model="newProviderType"
              class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white focus:outline-none focus:border-blue-500"
            >
              <option value="microsoft">Microsoft Entra</option>
              <!-- Add more provider types as needed -->
            </select>
          </div>
        </div>
        
        <div class="flex justify-end gap-3 mt-6">
          <button 
            @click="showAddProviderModal = false"
            class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 border border-slate-600"
          >
            Cancel
          </button>
          <button 
            @click="createProvider"
            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            :disabled="isLoading"
          >
            <span v-if="isLoading">Creating...</span>
            <span v-else>Create Provider</span>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Configuration Modal -->
    <div v-if="showConfigModal" class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center p-4 z-50">
      <div class="bg-slate-800 rounded-lg shadow-lg p-6 w-full max-w-lg border border-slate-700">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-semibold text-white">
            Configure {{ selectedProvider?.name }}
          </h2>
          <button @click="showConfigModal = false" class="text-slate-400 hover:text-white">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        <div v-if="selectedProvider?.provider_type === 'microsoft'" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-1">
              Client ID
            </label>
            <input 
              v-model="clientId"
              type="text"
              class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
              placeholder="Enter client ID"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-1">
              Tenant ID
            </label>
            <input 
              v-model="tenantId"
              type="text"
              class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
              placeholder="Enter tenant ID"
            />
          </div>
          
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-1">
              Client Secret
            </label>
            <input 
              v-model="clientSecret"
              type="password"
              class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
              placeholder="Enter new client secret (leave empty to keep existing)"
            />
            <p class="text-xs text-slate-400 mt-1">
              Only enter a value if you want to change the existing secret
            </p>
          </div>
          
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-1">
              Redirect URI
            </label>
            <input 
              v-model="redirectUri"
              type="text"
              class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-white placeholder-slate-400 focus:outline-none focus:border-blue-500"
              placeholder="Callback URL for authentication"
            />
          </div>
        </div>
        
        <div class="flex justify-end gap-3 mt-6">
          <button 
            @click="showConfigModal = false"
            class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 border border-slate-600"
          >
            Cancel
          </button>
          <button 
            @click="saveProviderConfig"
            class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
            :disabled="isLoading"
          >
            <span v-if="isLoading">Saving...</span>
            <span v-else>Save Configuration</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-providers-view {
  max-width: 1200px;
  margin: 0 auto;
}
</style> 