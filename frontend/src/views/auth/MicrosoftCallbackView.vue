<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import apiClient from '@/services/apiConfig';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const error = ref<string | null>(null);
const detailedError = ref<string | null>(null);
const loading = ref(true);
const message = ref('Completing sign-in...');

onMounted(async () => {
  // Extract the code and state parameters from the URL
  const code = route.query.code as string | undefined;
  const state = route.query.state as string | undefined;
  const errorParam = route.query.error as string | undefined;
  const errorDescription = route.query.error_description as string | undefined;
  
  // Log the received parameters for debugging
  console.log('Callback URL params:', { 
    code: code ? `${code.substring(0, 20)}...` : 'missing',
    state: state ? `${state.substring(0, 20)}...` : 'missing',
    error: errorParam,
    errorDescription
  });

  // Handle errors returned by Microsoft
  if (errorParam) {
    error.value = errorDescription || errorParam;
    loading.value = false;
    return;
  }

  // Validate that we have code and state
  if (!code || !state) {
    error.value = 'Missing required authentication parameters';
    detailedError.value = `Missing: ${!code ? 'code' : ''} ${!state ? 'state' : ''}`;
    loading.value = false;
    return;
  }

  try {
    // Call backend to handle the OAuth code exchange with the code and state parameters
    message.value = 'Processing authentication response...';
    console.log('Sending callback request to backend...');
    
    const response = await apiClient.get(`auth/oauth/callback`, {
      params: {
        code,
        state
      }
    });

    console.log('Backend response received:', response.status);
    
    // Store the token and user data
    const data = response.data;
    
    if (data && data.token) {
      message.value = 'Authentication successful, loading profile...';
      console.log('Authentication successful, token received');
      
      // Save token and user data in auth store using the new method
      // This will set the X-Auth-Provider header and fetch user data if needed
      await authStore.setExternalAuth(data.token, data.user || null, 'microsoft');
      
      // Redirect to dashboard or original destination
      let redirectPath = data.redirect || sessionStorage.getItem('authRedirect') || '/';
      // If the redirect is the callback URL, go to home instead
      if (redirectPath.includes('/auth/microsoft/callback')) {
        redirectPath = '/';
      }
      sessionStorage.removeItem('authRedirect'); // Clear stored redirect
      console.log('Redirecting to:', redirectPath);
      router.push(redirectPath);
    } else {
      console.error('Invalid response format:', data);
      error.value = 'Invalid response from server';
      detailedError.value = 'Server response did not contain a token. Response format: ' + 
                          JSON.stringify(data, null, 2).substring(0, 200);
      loading.value = false;
    }
  } catch (err: any) {
    console.error('Error during authentication:', err);
    
    // Try to extract error message from response if available
    const errorMsg = err.response?.data?.message || 
                    err.response?.data?.error || 
                    'An unexpected error occurred during authentication';
    
    // Get detailed error information
    let details = '';
    if (err.response) {
      details = `Status: ${err.response.status}\nData: ${JSON.stringify(err.response.data, null, 2)}\n`;
    } else if (err.request) {
      details = 'No response received from server';
    } else {
      details = err.message || 'Unknown error';
    }
    
    error.value = errorMsg;
    detailedError.value = details;
    loading.value = false;
  }
});
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-slate-900">
    <div class="bg-slate-800 p-8 rounded-lg shadow-lg max-w-md w-full">
      <div v-if="loading" class="flex flex-col items-center justify-center gap-4">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
        <h2 class="text-xl font-medium text-white">{{ message }}</h2>
        <p class="text-slate-400 text-center">Please wait while we complete your authentication</p>
      </div>
      
      <div v-else-if="error" class="flex flex-col items-center justify-center gap-4">
        <div class="bg-red-500 rounded-full p-3">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </div>
        <h2 class="text-xl font-medium text-white">Authentication Failed</h2>
        <p class="text-red-400 text-center">{{ error }}</p>
        
        <!-- Detailed Error Info (for debugging) -->
        <div v-if="detailedError" class="mt-4 w-full overflow-auto max-h-60 bg-slate-900 p-3 rounded text-xs text-slate-300 font-mono">
          <p class="text-red-400 mb-2 text-sm">Technical Details:</p>
          <pre>{{ detailedError }}</pre>
        </div>
        
        <button 
          @click="router.push('/login')" 
          class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Return to Login
        </button>
      </div>
    </div>
  </div>
</template> 