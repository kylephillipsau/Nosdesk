<script setup lang="ts">
import { ref, computed } from 'vue';
import ToggleSwitch from '@/components/common/ToggleSwitch.vue';

// MFA state
const mfaEnabled = ref(false);
const mfaStep = ref<'setup' | 'verify' | 'enabled'>('setup');
const qrCodeUrl = ref('');
const verificationCode = ref('');
const loading = ref(false);

// Emits for notifications
const emit = defineEmits<{
  (e: 'success', message: string): void;
  (e: 'error', message: string): void;
}>();

// Computed properties
const showSetupSteps = computed(() => !mfaEnabled.value && mfaStep.value === 'setup');
const showVerificationStep = computed(() => !mfaEnabled.value && mfaStep.value === 'verify');
const showEnabledState = computed(() => mfaEnabled.value);

// MFA functions
const toggleMFA = async () => {
  if (mfaEnabled.value) {
    // Disable MFA
    await disableMFA();
  } else {
    // Start MFA setup
    await startMFASetup();
  }
};

const startMFASetup = async () => {
  loading.value = true;
  try {
    // TODO: Implement MFA setup API call
    // const response = await authService.setupMFA();
    // qrCodeUrl.value = response.qrCode;
    qrCodeUrl.value = 'data:image/png;base64,placeholder'; // Placeholder
    mfaStep.value = 'verify';
    emit('success', 'MFA setup initiated. Please scan the QR code with your authenticator app.');
  } catch (err) {
    emit('error', 'Failed to setup MFA');
    console.error('Error setting up MFA:', err);
  } finally {
    loading.value = false;
  }
};

const verifyMFA = async () => {
  if (verificationCode.value.length !== 6) {
    emit('error', 'Please enter a valid 6-digit code');
    return;
  }

  loading.value = true;
  try {
    // TODO: Implement MFA verification API call
    // await authService.verifyMFA(verificationCode.value);
    mfaEnabled.value = true;
    mfaStep.value = 'enabled';
    verificationCode.value = '';
    emit('success', 'MFA enabled successfully! Your account is now more secure.');
  } catch (err) {
    emit('error', 'Invalid verification code. Please try again.');
    console.error('Error verifying MFA:', err);
  } finally {
    loading.value = false;
  }
};

const disableMFA = async () => {
  loading.value = true;
  try {
    // TODO: Implement MFA disable API call
    // await authService.disableMFA();
    mfaEnabled.value = false;
    mfaStep.value = 'setup';
    emit('success', 'MFA disabled successfully');
  } catch (err) {
    emit('error', 'Failed to disable MFA');
    console.error('Error disabling MFA:', err);
  } finally {
    loading.value = false;
  }
};

const cancelMFASetup = () => {
  mfaStep.value = 'setup';
  qrCodeUrl.value = '';
  verificationCode.value = '';
};
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <h2 class="text-lg font-medium text-white">Two-Factor Authentication</h2>
      <p class="text-sm text-slate-400 mt-1">Add an extra layer of security to your account</p>
    </div>
    
    <div class="p-6">
      <!-- MFA Toggle / Status -->
      <div class="mb-6">
        <ToggleSwitch
          v-model="mfaEnabled"
          :disabled="loading"
          label="Enable Two-Factor Authentication"
          :description="mfaEnabled ? 'Your account is protected with 2FA' : 'Secure your account with an authenticator app'"
          @update:modelValue="toggleMFA"
        />
      </div>

      <!-- Setup Steps -->
      <div v-if="showSetupSteps" class="flex flex-col gap-3">
        <div class="bg-slate-700/30 rounded-lg p-4">
          <h3 class="text-sm font-medium text-white mb-2">How to set up 2FA:</h3>
                      <ol class="flex flex-col gap-2 text-sm text-slate-300">
            <li class="flex items-start gap-2">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5">1</span>
              <span>Download an authenticator app like Google Authenticator or Authy</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5">2</span>
              <span>Click the toggle above to start the setup process</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5">3</span>
              <span>Scan the QR code with your authenticator app</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5">4</span>
              <span>Enter the 6-digit code to complete setup</span>
            </li>
          </ol>
        </div>
      </div>

      <!-- Verification Step -->
      <div v-if="showVerificationStep" class="flex flex-col gap-3">
        <div class="bg-slate-700/30 rounded-lg p-4">
          <h3 class="text-sm font-medium text-white mb-2">Scan QR Code</h3>
          <p class="text-sm text-slate-400 mb-3">Scan this QR code with your authenticator app:</p>
          
          <!-- QR Code placeholder -->
          <div class="bg-white p-4 rounded-lg mb-4 inline-block">
            <div class="w-32 h-32 bg-gray-200 flex items-center justify-center text-gray-500 text-xs">
              QR Code
            </div>
          </div>
          
          <h4 class="text-sm font-medium text-white mb-2">Enter Verification Code</h4>
          <p class="text-sm text-slate-400 mb-3">Enter the 6-digit code from your authenticator app:</p>
          
          <div class="flex gap-3">
            <div class="bg-slate-700/50 rounded-lg border border-slate-600/30 flex-1">
              <input
                v-model="verificationCode"
                type="text"
                maxlength="6"
                class="w-full px-4 py-2 bg-transparent text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none text-center tracking-widest"
                placeholder="000000"
              />
            </div>
            <button
              @click="verifyMFA"
              :disabled="verificationCode.length !== 6 || loading"
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center"
            >
              <span v-if="loading" class="animate-spin h-4 w-4 mr-2">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 718-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              </span>
              Verify
            </button>
          </div>
          
          <button
            @click="cancelMFASetup"
            class="mt-3 text-sm text-slate-400 hover:text-white transition-colors"
          >
            Cancel setup
          </button>
        </div>
      </div>

      <!-- Enabled State -->
      <div v-if="showEnabledState" class="flex flex-col gap-3">
        <div class="bg-green-600/10 border border-green-600/20 rounded-lg p-4">
          <div class="flex items-center gap-2 mb-2">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <h3 class="text-sm font-medium text-green-400">2FA is enabled</h3>
          </div>
          <p class="text-sm text-slate-300">Your account is protected with two-factor authentication. You'll need to enter a code from your authenticator app when signing in.</p>
        </div>
      </div>
    </div>
  </div>
</template> 