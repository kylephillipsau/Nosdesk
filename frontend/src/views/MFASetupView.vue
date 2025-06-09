<template>
  <div class="w-full min-h-screen bg-slate-900 flex justify-center">
    <div class="flex flex-col gap-6 w-full max-w-4xl p-8">
      <!-- Header -->
      <div class="flex flex-col gap-2 items-center">
        <img :src="logo" alt="Nosdesk Logo" class="px-4 max-w-md" />
        <h1 class="text-2xl font-bold text-white mt-4">Complete Your Account Setup</h1>
        <p class="text-slate-400 text-center">
          Your account type requires multi-factor authentication for security
        </p>
      </div>

      <!-- Error Message -->
      <div v-if="errorMessage" class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg text-sm">
        {{ errorMessage }}
      </div>

      <!-- Success Message -->
      <div v-if="successMessage" class="bg-green-900/50 border border-green-700 text-green-200 px-4 py-3 rounded-lg text-sm">
        {{ successMessage }}
      </div>

      <!-- Info Banner -->
      <div class="bg-blue-900/50 border border-blue-700 text-blue-200 px-4 py-3 rounded-lg text-sm">
        <div class="flex items-center gap-2">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
          Once you complete MFA setup, you'll be automatically signed in to your account.
        </div>
      </div>

      <!-- MFA Settings Component - Reusing existing component -->
      <div class="bg-slate-800 rounded-xl border border-slate-700/50">
        <MFASettings 
          ref="mfaSettingsRef"
          :is-login-setup="true"
          @success="handleMfaSetupSuccess" 
          @error="handleMfaSetupError"
        />
      </div>

      <!-- Navigation -->
      <div class="flex justify-between items-center">
        <button
          @click="goBackToLogin"
          class="flex items-center gap-2 px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
          </svg>
          Back to Login
        </button>
        
        <div class="text-xs text-slate-500">
          Step 1 of 1: Setup Multi-Factor Authentication
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import MFASettings from '@/components/settings/MFASettings.vue';
import logo from '@/assets/logo.svg';

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const errorMessage = ref('');
const successMessage = ref('');

// Template ref for MFASettings component
const mfaSettingsRef = ref();

// Get credentials from navigation state (more secure than URL params)
const credentials = ref<{ email: string; password: string } | null>(null);

// Security check and credential extraction
onMounted(async () => {
  // Check if we have the proper navigation state
  if (history.state?.email && history.state?.password && history.state?.from === 'login') {
    credentials.value = {
      email: history.state.email,
      password: history.state.password
    };
    
    // Verify the credentials are valid and user actually needs MFA setup
    try {
      const setupData = await authStore.startMfaSetupLogin(
        credentials.value.email, 
        credentials.value.password
      );
      
      if (!setupData) {
        throw new Error('Invalid credentials or MFA setup not required');
      }
      
      // Store setup context for the MFASettings component to use
      sessionStorage.setItem('mfaLoginSetupContext', JSON.stringify({
        email: credentials.value.email,
        password: credentials.value.password,
        setupData,
        timestamp: Date.now()
      }));
      
      // Now start the MFA setup in the child component
      if (mfaSettingsRef.value) {
        await mfaSettingsRef.value.startMFASetup();
      }
      
    } catch (error: any) {
      console.error('MFA setup validation error:', error);
      errorMessage.value = 'Unable to verify credentials. Please try again.';
      setTimeout(() => {
        router.push('/login');
      }, 3000);
    }
  } else {
    // No proper credentials - redirect back to login
    errorMessage.value = 'Invalid access. Redirecting to login...';
    setTimeout(() => {
      router.push('/login');
    }, 2000);
  }
});

// Handle successful MFA setup from the component
const handleMfaSetupSuccess = async (message: string) => {
  successMessage.value = message;
  
  // Only proceed with automatic login if MFA is actually enabled (not just initiated)
  if (message.includes('MFA enabled successfully')) {
    // Clean up the temporary session data
    sessionStorage.removeItem('mfaLoginSetupContext');
    
    // Complete the login process with the stored credentials
    if (credentials.value) {
      try {
        // Clear any previous error messages
        errorMessage.value = '';
        
        // The user now has MFA enabled, so we can complete normal login
        const loginSuccess = await authStore.login({
          email: credentials.value.email,
          password: credentials.value.password
        });
        
        if (loginSuccess) {
          // Login successful - redirect to dashboard
          router.push('/');
        } else {
          // This shouldn't happen, but handle gracefully
          errorMessage.value = 'MFA setup completed, but login failed. Please try logging in manually.';
          setTimeout(() => {
            router.push('/login');
          }, 3000);
        }
      } catch (error) {
        console.error('Post-MFA login error:', error);
        errorMessage.value = 'MFA setup completed successfully. Please log in manually.';
        setTimeout(() => {
          router.push('/login');
        }, 3000);
      }
    }
  }
  // If it's just "MFA setup initiated", don't do anything - let user complete the process
};

// Handle MFA setup errors from the component
const handleMfaSetupError = (error: string) => {
  errorMessage.value = error;
};

// Navigation back to login
const goBackToLogin = () => {
  // Clean up any temporary data
  sessionStorage.removeItem('mfaLoginSetupContext');
  authStore.clearMfaState();
  router.push('/login');
};
</script> 