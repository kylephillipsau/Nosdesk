<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';
import BackButton from '@/components/common/BackButton.vue';

// Define types for our data structures
interface Provider {
  id: number;
  name: string;
  provider_type: string;
  enabled: boolean;
  is_default: boolean;
}

interface ConfigValidation {
  valid: boolean;
  client_id?: string;
  tenant_id?: string;
  client_secret_configured?: boolean;
  redirect_uri?: string;
  error?: string;
}

const router = useRouter();

// State for providers
const isLoading = ref(false);
const errorMessage = ref('');
const providers = ref<Provider[]>([]);
const successMessage = ref('');
const testingProvider = ref<number | null>(null);
const configValidations = ref<Record<number, ConfigValidation>>({});

// Load providers from API
const loadProviders = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    const response = await axios.get('/api/admin/auth/providers');
    providers.value = response.data;
  } catch (error: any) {
    console.error('Failed to load auth providers:', error);
    errorMessage.value = error.response?.data?.message || 'Failed to load authentication providers';
  } finally {
    isLoading.value = false;
  }
};

// Set a provider as default
const setAsDefault = async (provider: Provider) => {
  try {
    await axios.post('/api/admin/auth/providers/default', { provider_id: provider.id });
    
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

// Test provider configuration
const testProviderConfig = async (provider: Provider) => {
  testingProvider.value = provider.id;
  
  try {
    if (provider.provider_type === 'microsoft') {
      // Use the configuration validation endpoint instead of full OAuth
      const response = await axios.get(`/api/integrations/graph/config`);
      
      configValidations.value[provider.id] = {
        valid: true,
        client_id: response.data.client_id,
        tenant_id: response.data.tenant_id,
        client_secret_configured: response.data.client_secret_configured,
        redirect_uri: response.data.redirect_uri
      };
      
      successMessage.value = `${provider.name} configuration is valid`;
    } else {
      // For other providers, we could add similar validation endpoints
      successMessage.value = `${provider.name} configuration check not implemented`;
    }
    
    setTimeout(() => { successMessage.value = ''; }, 3000);
  } catch (error: any) {
    console.error(`Error testing ${provider.name} configuration:`, error);
    
    const errorDetails = error.response?.data;
    configValidations.value[provider.id] = {
      valid: false,
      error: errorDetails?.message || `Configuration validation failed`
    };
    
    errorMessage.value = `${provider.name} configuration error: ${errorDetails?.message || 'Configuration validation failed'}`;
    setTimeout(() => { errorMessage.value = ''; }, 3000);
  } finally {
    testingProvider.value = null;
  }
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
    case 'local':
      return `
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
      `;
    default:
      return `
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
      `;
  }
};

// Helper to get provider status description
const getProviderDescription = (provider: Provider) => {
  switch (provider.provider_type) {
    case 'microsoft':
      return 'Microsoft Entra ID (Azure AD) Single Sign-On';
    case 'local':
      return 'Email and password authentication';
    case 'google':
      return 'Google Workspace Single Sign-On';
    default:
      return 'External authentication provider';
  }
};

// Helper to get configuration requirements
const getConfigRequirements = (provider: Provider) => {
  switch (provider.provider_type) {
    case 'microsoft':
      return [
        'MICROSOFT_CLIENT_ID',
        'MICROSOFT_CLIENT_SECRET', 
        'MICROSOFT_TENANT_ID',
        'MICROSOFT_REDIRECT_URI'
      ];
    case 'google':
      return [
        'GOOGLE_CLIENT_ID',
        'GOOGLE_CLIENT_SECRET',
        'GOOGLE_REDIRECT_URI'
      ];
    default:
      return [];
  }
};

onMounted(() => {
  loadProviders();
});
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div class="pt-4 px-6 flex justify-between items-center">
      <BackButton fallbackRoute="/admin/settings" label="Back to Administration" />
    </div>
    
    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
      <div class="mb-6">
        <h1 class="text-2xl font-bold text-white">Authentication Providers</h1>
        <p class="text-slate-400 mt-2">
          View authentication provider status and validate configuration. Providers are configured via environment variables.
        </p>
      </div>
      
      <!-- Configuration Notice -->
      <div class="p-4 bg-blue-900/30 text-blue-400 rounded-xl border border-blue-700/50 mb-4 flex items-start">
        <div class="mr-3 mt-0.5 text-blue-400 flex-shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
        </div>
        <div>
          <p class="font-medium">Configuration via Environment Variables</p>
          <p class="text-sm text-blue-300 mt-1">
            Authentication providers are configured through environment variables in your 
            <code class="bg-blue-800/50 px-1 rounded">.env</code> file. 
            Use the "Validate Config" button to check if each provider is properly configured.
          </p>
        </div>
      </div>
      
      <!-- Success message -->
      <div 
        v-if="successMessage" 
        class="p-4 bg-green-900/30 text-green-400 rounded-xl border border-green-700/50 mb-4 flex items-start"
      >
        <div class="mr-3 mt-0.5 text-green-400 flex-shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
          </svg>
        </div>
        <div>{{ successMessage }}</div>
      </div>
      
      <!-- Error message -->
      <div 
        v-if="errorMessage" 
        class="p-4 bg-red-900/30 text-red-400 rounded-xl border border-red-700/50 mb-4 flex items-start"
      >
        <div class="mr-3 mt-0.5 text-red-400 flex-shrink-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </div>
        <div>{{ errorMessage }}</div>
      </div>
      
      <!-- Loading state -->
      <div v-if="isLoading" class="flex justify-center my-8">
        <div class="animate-spin h-8 w-8 text-blue-500">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
        <span class="ml-3 text-white">Loading providers...</span>
      </div>
      
      <!-- Provider list -->
      <div v-else class="flex flex-col gap-4">
        <div v-for="provider in providers" :key="provider.id" 
             class="bg-slate-800 border border-slate-700/50 rounded-xl hover:border-slate-600/50 transition-colors">
          
          <!-- Provider Header -->
          <div class="p-6 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
            <div class="flex items-start gap-4">
              <!-- Provider icon -->
              <div class="flex-shrink-0 h-12 w-12 rounded-lg bg-slate-700/50 flex items-center justify-center text-slate-300" v-html="getProviderIcon(provider.provider_type)"></div>
              
              <!-- Provider name and status -->
              <div>
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="font-medium text-lg text-white">{{ provider.name }}</span>
                  <span v-if="provider.is_default" 
                        class="px-2 py-0.5 text-xs bg-blue-900/50 text-blue-200 rounded-full border border-blue-700">
                    Default
                  </span>
                  <span 
                    class="px-2 py-0.5 text-xs rounded-full border" 
                    :class="provider.enabled ? 'bg-green-900/50 text-green-200 border-green-700' : 'bg-slate-700/50 text-slate-400 border-slate-600'"
                  >
                    {{ provider.enabled ? 'Available' : 'Not Configured' }}
                  </span>
                </div>
                <div class="text-sm text-slate-400 mt-1">
                  {{ getProviderDescription(provider) }}
                </div>
                
                <!-- Configuration requirements -->
                <div v-if="getConfigRequirements(provider).length > 0" class="mt-2">
                  <div class="text-xs text-slate-500 mb-1">Required environment variables:</div>
                  <div class="flex flex-wrap gap-1">
                    <code 
                      v-for="envVar in getConfigRequirements(provider)" 
                      :key="envVar"
                      class="text-xs bg-slate-700/50 text-slate-300 px-1.5 py-0.5 rounded"
                    >
                      {{ envVar }}
                    </code>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Provider actions -->
            <div class="flex flex-wrap gap-2 ml-auto">
              <button 
                v-if="provider.enabled && !provider.is_default"
                @click="setAsDefault(provider)"
                class="px-3 py-1.5 bg-blue-900/50 text-blue-200 rounded-lg text-sm hover:bg-blue-800/50 border border-blue-700 font-medium transition-colors"
              >
                Set as Default
              </button>
              
              <button 
                v-if="provider.enabled && provider.provider_type !== 'local'"
                @click="testProviderConfig(provider)"
                :disabled="testingProvider === provider.id"
                class="px-3 py-1.5 bg-purple-900/50 text-purple-200 rounded-lg text-sm hover:bg-purple-800/50 border border-purple-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                <svg v-if="testingProvider === provider.id" class="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                {{ testingProvider === provider.id ? 'Testing...' : 'Validate Config' }}
              </button>
            </div>
          </div>
          
          <!-- Configuration Validation Results -->
          <div v-if="configValidations[provider.id]" class="border-t border-slate-700/50 p-4 bg-slate-700/20">
            <div v-if="configValidations[provider.id].valid" class="text-sm">
              <div class="flex items-center gap-2 text-green-400 mb-2">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                </svg>
                Configuration Valid
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs text-slate-300">
                <div v-if="configValidations[provider.id].client_id">
                  <span class="text-slate-400">Client ID:</span> {{ configValidations[provider.id].client_id }}
                </div>
                <div v-if="configValidations[provider.id].tenant_id">
                  <span class="text-slate-400">Tenant ID:</span> {{ configValidations[provider.id].tenant_id }}
                </div>
                <div v-if="configValidations[provider.id].client_secret_configured !== undefined">
                  <span class="text-slate-400">Client Secret:</span> 
                  <span :class="configValidations[provider.id].client_secret_configured ? 'text-green-400' : 'text-red-400'">
                    {{ configValidations[provider.id].client_secret_configured ? 'Configured' : 'Not Configured' }}
                  </span>
                </div>
                <div v-if="configValidations[provider.id].redirect_uri">
                  <span class="text-slate-400">Redirect URI:</span> {{ configValidations[provider.id].redirect_uri }}
                </div>
              </div>
            </div>
            <div v-else class="text-sm">
              <div class="flex items-center gap-2 text-red-400 mb-1">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
                Configuration Error
              </div>
              <div class="text-xs text-red-300">{{ configValidations[provider.id].error }}</div>
            </div>
          </div>
        </div>
        
        <div v-if="providers.length === 0 && !isLoading" class="text-center py-12 text-slate-400 bg-slate-800 rounded-xl border border-slate-700/50 p-6">
          <div class="flex justify-center mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <p class="text-lg font-medium">No authentication providers found</p>
          <p class="mt-2 text-slate-500">Configure authentication providers in your environment variables</p>
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