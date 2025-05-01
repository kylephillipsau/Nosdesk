<!-- LoginView.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import logo from '@/assets/logo.svg';
import axios from 'axios';

const router = useRouter();
const authStore = useAuthStore();
const email = ref('');
const password = ref('');
const rememberMe = ref(false);
const isLoading = ref(false);
const errorMessage = ref('');

const handleLogin = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    const success = await authStore.login({
      email: email.value,
      password: password.value
    });
    
    if (!success && authStore.error) {
      errorMessage.value = authStore.error;
    }
  } catch (error) {
    console.error('Login error:', error);
    errorMessage.value = 'An unexpected error occurred. Please try again.';
  } finally {
    isLoading.value = false;
  }
};

const handleMicrosoftLogin = async () => {
  isLoading.value = true;
  errorMessage.value = '';
  
  try {
    // Store the current URL to redirect back after authentication
    const redirectPath = router.currentRoute.value.query.redirect?.toString() || '/';
    sessionStorage.setItem('authRedirect', redirectPath);
    
    // Get authorization URL from backend
    const response = await axios.post('/api/auth/oauth/authorize', {
      provider_type: 'microsoft',
      redirect_uri: `${window.location.origin}/auth/microsoft/callback`
    });
    
    // Make sure we got a valid auth URL
    if (response.data && response.data.auth_url) {
      // Redirect to Microsoft login
      window.location.href = response.data.auth_url;
    } else {
      throw new Error('Invalid authorization URL received');
    }
  } catch (error: any) {
    console.error('Error initiating Microsoft authentication:', error);
    errorMessage.value = error.response?.data?.message || 
                          error.response?.data?.error ||
                          'Failed to initiate Microsoft authentication';
    isLoading.value = false;
  }
};

const handleMicrosoftLogout = async () => {
  try {
    errorMessage.value = '';
    
    // Get the sign-out URL from backend
    const response = await axios.post('/api/auth/oauth/logout', {
      provider_type: 'microsoft',
      redirect_uri: window.location.href
    });
    
    // Redirect to Microsoft logout page
    if (response.data && response.data.logout_url) {
      window.location.href = response.data.logout_url;
    } else {
      throw new Error('Invalid logout URL received');
    }
  } catch (error: any) {
    console.error('Error logging out of Microsoft:', error);
    errorMessage.value = error.response?.data?.message || 
                        'Failed to initiate Microsoft logout';
  }
};
</script>

<template>
  <div class="min-h-screen w-full flex items-center justify-center bg-slate-900">
    <div class="flex flex-col gap-4 w-full max-w-md p-8">
      <!-- Logo/Brand -->
      <div class="flex flex-col gap-2 items-center">
        <img :src="logo" alt="Nosdesk Logo" class="px-8" />
        <p class="text-slate-400 mt-2">Sign in to your account</p>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg text-sm">
        {{ errorMessage }}
      </div>

      <!-- Login Form -->
      <form @submit.prevent="handleLogin" class="flex flex-col gap-4 space-y-6">
        <div>
          <label for="email" class="block text-sm font-medium text-slate-300">Email</label>
          <input
            id="email"
            v-model="email"
            type="email"
            required
            class="mt-1 block w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            placeholder="Enter your email"
          />
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-slate-300">Password</label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            class="mt-1 block w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            placeholder="Enter your password"
          />
        </div>

        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5">
            <input
              id="remember-me"
              v-model="rememberMe"
              type="checkbox"
              class="h-4 w-4 rounded border-slate-700 bg-slate-800 text-blue-500 focus:ring-blue-500 focus:ring-offset-slate-900"
            />
            <label for="remember-me" class="ml-2 block text-sm text-slate-300">Remember me</label>
          </div>

          <a href="#" class="text-sm text-blue-500 hover:text-blue-400">Forgot password?</a>
        </div>

        <button
          type="submit"
          :disabled="isLoading"
          class="w-full flex justify-center py-2 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 focus:ring-offset-slate-900 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <span v-if="isLoading">Signing in...</span>
          <span v-else>Sign in</span>
        </button>
        
        <div class="relative flex items-center justify-center">
          <div class="border-t border-slate-700 flex-grow"></div>
          <span class="mx-4 text-sm text-slate-500">or</span>
          <div class="border-t border-slate-700 flex-grow"></div>
        </div>
        
        <div class="flex gap-2">
          <button
            type="button"
            @click="handleMicrosoftLogin"
            class="flex-1 flex gap-1 justify-center items-center py-2 px-4 border border-slate-600 rounded-lg shadow-sm text-sm font-medium text-slate-200 bg-slate-800 hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-slate-500 focus:ring-offset-slate-900"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 21 21" class="mr-2">
              <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
              <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
              <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
              <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
            </svg>
            Sign in with Microsoft Entra
          </button>
          
          <button
            type="button"
            @click="handleMicrosoftLogout"
            title="Sign out of Microsoft account"
            class="p-2 border border-slate-600 rounded-lg text-slate-400 bg-slate-800 hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-slate-500 focus:ring-offset-slate-900"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M3 3a1 1 0 00-1 1v12a1 1 0 102 0V4a1 1 0 00-1-1zm10.293 9.293a1 1 0 001.414 1.414l3-3a1 1 0 000-1.414l-3-3a1 1 0 10-1.414 1.414L14.586 9H7a1 1 0 100 2h7.586l-1.293 1.293z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
