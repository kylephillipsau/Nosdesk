<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

// State for providers
const isLoading = ref(false);
const errorMessage = ref('');
const providers = ref([
  {
    id: 'local',
    name: 'Local Authentication',
    description: 'Username and password authentication',
    enabled: true,
    isDefault: true,
    icon: 'user',
    configurable: false,
    requiresExternal: false
  },
  {
    id: 'microsoft',
    name: 'Microsoft Entra ID',
    description: 'Single sign-on with Microsoft Entra ID (formerly Azure AD)',
    enabled: false,
    isDefault: false,
    icon: 'microsoft',
    configurable: true,
    requiresExternal: true,
    config: {
      clientId: '',
      tenantId: '',
      redirectUri: `${window.location.origin}/auth/microsoft/callback`
    }
  },
  {
    id: 'google',
    name: 'Google Workspace',
    description: 'Single sign-on with Google Workspace accounts',
    enabled: false,
    isDefault: false,
    icon: 'google',
    configurable: true,
    requiresExternal: true
  },
  {
    id: 'saml',
    name: 'SAML 2.0',
    description: 'Enterprise SSO via SAML protocol',
    enabled: false,
    isDefault: false,
    icon: 'key',
    configurable: true,
    requiresExternal: true
  }
]);

// Selected provider for configuration
const selectedProvider = ref<any>(null);
const showConfigModal = ref(false);

// Client ID and Tenant ID for Microsoft Entra
const clientId = ref('');
const tenantId = ref('');
const clientSecret = ref('');
const redirectUri = ref(`${window.location.origin}/auth/microsoft/callback`);

// Mock function to load providers from API
const loadProviders = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    // In a real implementation, this would fetch from your backend
    // const response = await fetch('/api/auth/providers');
    // providers.value = await response.json();
    
    // For now, we're using mock data initialized above
    await new Promise(resolve => setTimeout(resolve, 500)); // Simulate network delay
  } catch (error) {
    console.error('Failed to load auth providers:', error);
    errorMessage.value = 'Failed to load authentication providers';
  } finally {
    isLoading.value = false;
  }
};

// Toggle provider enabled state
const toggleProvider = async (provider: any) => {
  const updatedProvider = { ...provider, enabled: !provider.enabled };
  
  try {
    // In a real implementation, you would call your API
    // await fetch(`/api/auth/providers/${provider.id}`, {
    //   method: 'PATCH',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify({ enabled: updatedProvider.enabled })
    // });
    
    // Update the provider in the local state
    const index = providers.value.findIndex(p => p.id === provider.id);
    if (index !== -1) {
      providers.value[index] = updatedProvider;
    }
  } catch (error) {
    console.error(`Failed to update ${provider.name}:`, error);
    // Revert the local state change on error
    const index = providers.value.findIndex(p => p.id === provider.id);
    if (index !== -1) {
      providers.value[index] = provider;
    }
  }
};

// Open configuration modal for a provider
const configureProvider = (provider: any) => {
  selectedProvider.value = provider;
  
  // Initialize form values from provider config
  if (provider.id === 'microsoft' && provider.config) {
    clientId.value = provider.config.clientId || '';
    tenantId.value = provider.config.tenantId || '';
    clientSecret.value = ''; // We don't store or display secrets
    redirectUri.value = provider.config.redirectUri || `${window.location.origin}/auth/microsoft/callback`;
  }
  
  showConfigModal.value = true;
};

// Save provider configuration
const saveProviderConfig = async () => {
  if (!selectedProvider.value) return;
  
  try {
    // Update local config
    if (selectedProvider.value.id === 'microsoft') {
      selectedProvider.value.config = {
        clientId: clientId.value,
        tenantId: tenantId.value,
        redirectUri: redirectUri.value
      };
      
      // In a real implementation, you would call your API
      // await fetch(`/api/auth/providers/${selectedProvider.value.id}/config`, {
      //   method: 'PUT',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify({
      //     clientId: clientId.value,
      //     tenantId: tenantId.value,
      //     clientSecret: clientSecret.value || undefined,
      //     redirectUri: redirectUri.value
      //   })
      // });
      
      // Enable the provider if it's being configured for the first time
      if (!selectedProvider.value.enabled) {
        selectedProvider.value.enabled = true;
      }
      
      // Update the provider in the local state
      const index = providers.value.findIndex(p => p.id === selectedProvider.value.id);
      if (index !== -1) {
        providers.value[index] = selectedProvider.value;
      }
    }
    
    // Close the modal
    showConfigModal.value = false;
  } catch (error) {
    console.error(`Failed to save ${selectedProvider.value.name} configuration:`, error);
  }
};

// Set a provider as default
const setAsDefault = async (provider: any) => {
  try {
    // In a real implementation, you would call your API
    // await fetch(`/api/auth/providers/default`, {
    //   method: 'PUT',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify({ providerId: provider.id })
    // });
    
    // Update local state
    providers.value.forEach(p => {
      p.isDefault = p.id === provider.id;
    });
  } catch (error) {
    console.error(`Failed to set ${provider.name} as default:`, error);
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

onMounted(() => {
  loadProviders();
});
</script>

<template>
  <div class="p-4 md:p-6 max-w-7xl mx-auto">
    <div class="flex items-center mb-6">
      <button 
        @click="goBack" 
        class="mr-4 text-slate-400 hover:text-white"
        aria-label="Go back"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
      </button>
      <div>
        <h1 class="text-2xl font-bold text-white">Authentication Providers</h1>
        <p class="text-slate-400 mt-1">
          Configure how users sign in to your application
        </p>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="isLoading" class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
    </div>

    <!-- Error message -->
    <div v-else-if="errorMessage" class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg">
      {{ errorMessage }}
      <button 
        @click="loadProviders" 
        class="ml-2 underline hover:text-white"
      >
        Retry
      </button>
    </div>

    <!-- Providers list -->
    <div v-else class="space-y-4">
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="bg-slate-800 border border-slate-700 rounded-lg overflow-hidden"
      >
        <div class="p-5">
          <div class="flex items-start justify-between">
            <div class="flex items-start">
              <div class="flex-shrink-0 h-10 w-10 rounded-md bg-blue-600/20 flex items-center justify-center text-blue-400 mr-4">
                <div v-html="getProviderIcon(provider.icon)"></div>
              </div>
              <div>
                <div class="flex items-center">
                  <h3 class="text-white font-medium">{{ provider.name }}</h3>
                  <span v-if="provider.isDefault" class="ml-2 px-2 py-0.5 text-xs rounded-full bg-green-900/50 text-green-400">Default</span>
                </div>
                <p class="mt-1 text-sm text-slate-400">{{ provider.description }}</p>
              </div>
            </div>
            <div class="flex items-center space-x-3">
              <button
                v-if="provider.configurable && provider.enabled"
                @click="setAsDefault(provider)"
                :disabled="provider.isDefault"
                class="px-3 py-1.5 text-xs rounded-lg border border-slate-600 bg-slate-700 text-white hover:bg-slate-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Set as Default
              </button>
              <button
                v-if="provider.configurable"
                @click="configureProvider(provider)"
                class="px-3 py-1.5 text-xs rounded-lg border border-slate-600 bg-slate-700 text-white hover:bg-slate-600"
              >
                Configure
              </button>
              <label class="relative inline-flex items-center cursor-pointer">
                <input 
                  type="checkbox" 
                  :checked="provider.enabled"
                  @change="toggleProvider(provider)"
                  class="sr-only peer"
                >
                <div class="w-11 h-6 bg-slate-700 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-slate-400 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 peer-checked:after:bg-white"></div>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Configuration Modal -->
    <div v-if="showConfigModal" class="fixed inset-0 bg-slate-900/80 flex items-center justify-center z-50 p-4">
      <div class="bg-slate-800 rounded-lg shadow-xl max-w-lg w-full max-h-[90vh] overflow-y-auto">
        <div class="p-5 border-b border-slate-700">
          <h3 class="text-xl font-semibold text-white">
            Configure {{ selectedProvider?.name }}
          </h3>
        </div>
        
        <div class="p-5">
          <!-- Microsoft Entra ID Configuration Form -->
          <form v-if="selectedProvider?.id === 'microsoft'" @submit.prevent="saveProviderConfig" class="space-y-4">
            <div>
              <label for="clientId" class="block text-sm font-medium text-slate-300 mb-1">Application (client) ID</label>
              <input
                id="clientId"
                v-model="clientId"
                type="text"
                required
                class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
                placeholder="Enter your Application ID from Microsoft Entra"
              />
            </div>
            
            <div>
              <label for="tenantId" class="block text-sm font-medium text-slate-300 mb-1">Directory (tenant) ID</label>
              <input
                id="tenantId"
                v-model="tenantId"
                type="text"
                required
                class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
                placeholder="Enter your Tenant ID from Microsoft Entra"
              />
            </div>
            
            <div>
              <label for="clientSecret" class="block text-sm font-medium text-slate-300 mb-1">Client Secret</label>
              <input
                id="clientSecret"
                v-model="clientSecret"
                type="password"
                class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
                placeholder="Enter client secret (only needed if changing)"
              />
              <p class="mt-1 text-xs text-slate-400">
                Leave blank to keep existing secret unchanged
              </p>
            </div>
            
            <div>
              <label for="redirectUri" class="block text-sm font-medium text-slate-300 mb-1">Redirect URI</label>
              <input
                id="redirectUri"
                v-model="redirectUri"
                type="text"
                required
                class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
              />
              <p class="mt-1 text-xs text-slate-400">
                Add this URL to the Redirect URIs in your Microsoft Entra app registration
              </p>
            </div>
            
            <div class="mt-6 bg-blue-900/30 border border-blue-800 rounded-lg p-3 text-sm text-blue-200">
              <p>
                <strong>Note:</strong> You need to register an app in the Microsoft Entra admin center
                and add the redirect URI shown above to your app registration.
              </p>
            </div>
          </form>
          
          <!-- Other provider configurations would go here -->
          <div v-else class="text-center py-8 text-slate-400">
            Configuration for {{ selectedProvider?.name }} is not yet implemented
          </div>
        </div>
        
        <div class="p-5 border-t border-slate-700 flex justify-end space-x-3">
          <button
            @click="showConfigModal = false"
            class="px-4 py-2 rounded-lg border border-slate-600 text-slate-300 hover:bg-slate-700"
          >
            Cancel
          </button>
          <button
            @click="saveProviderConfig"
            class="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
          >
            Save Configuration
          </button>
        </div>
      </div>
    </div>
  </div>
</template> 