<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import ToggleSwitch from '@/components/common/ToggleSwitch.vue';
import { useAuthStore } from '@/stores/auth';

// MFA state
const mfaEnabled = ref(false);
const mfaStep = ref<'setup' | 'verify' | 'enabled'>('setup');
const qrCodeUrl = ref('');
const verificationCode = ref('');
const loading = ref(false);
const backupCodes = ref<string[]>([]);
const mfaSecret = ref(''); // Store the secret temporarily during setup
const showSecret = ref(false); // Toggle for showing the secret text
const secretCopied = ref(false); // Track if secret was just copied

const authStore = useAuthStore();

// Props for different modes
const props = defineProps<{
  isLoginSetup?: boolean; // When true, this is for login-time MFA setup
}>();

// Emits for notifications
const emit = defineEmits<{
  (e: 'success', message: string): void;
  (e: 'error', message: string): void;
}>();

// Computed properties
const showSetupSteps = computed(() => !mfaEnabled.value && mfaStep.value === 'setup');
const showVerificationStep = computed(() => !mfaEnabled.value && mfaStep.value === 'verify');
const showEnabledState = computed(() => mfaEnabled.value);

// Check MFA status on component mount
onMounted(async () => {
  if (!props.isLoginSetup) {
    // In normal settings mode, check current status
    await checkMFAStatus();
  }
  // In login setup mode, wait for parent to call startMFASetup explicitly
});

// Check current MFA status
const checkMFAStatus = async () => {
  try {
    const response = await fetch('/api/auth/mfa/status', {
      headers: {
        'Authorization': `Bearer ${authStore.token}`,
        'Content-Type': 'application/json'
      }
    });

    if (response.ok) {
      const data = await response.json();
      mfaEnabled.value = data.enabled || false;
      if (mfaEnabled.value) {
        mfaStep.value = 'enabled';
      }
    }
  } catch (err) {
    console.error('Error checking MFA status:', err);
  }
};

// MFA functions
const toggleMFA = async (newValue: boolean) => {
  // Store the current state before any changes
  const currentState = mfaEnabled.value;
  
  if (currentState) {
    // Currently enabled, user wants to disable
    await disableMFA();
  } else {
    // Currently disabled, user wants to enable
    await startMFASetup();
  }
};

const startMFASetup = async () => {
  loading.value = true;
  try {
    let response;
    
    if (props.isLoginSetup) {
      // In login setup mode, use the stored context from the setup view
      const context = sessionStorage.getItem('mfaLoginSetupContext');
      if (!context) {
        throw new Error('MFA setup context not found');
      }
      
      const { email, password, setupData } = JSON.parse(context);
      
      // Use the setup data directly
      qrCodeUrl.value = setupData.qr_code;
      mfaSecret.value = setupData.secret;
      backupCodes.value = setupData.backup_codes || [];
      mfaStep.value = 'verify';
      emit('success', 'MFA setup initiated. Please scan the QR code with your authenticator app.');
      
    } else {
      // Normal authenticated setup mode
      response = await fetch('/api/auth/mfa/setup', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${authStore.token}`,
          'Content-Type': 'application/json'
        }
      });

      if (response.ok) {
        const data = await response.json();
        qrCodeUrl.value = data.qr_code;
        mfaSecret.value = data.secret;
        backupCodes.value = data.backup_codes || [];
        mfaStep.value = 'verify';
        emit('success', 'MFA setup initiated. Please scan the QR code with your authenticator app.');
      } else {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || 'Failed to setup MFA');
      }
    }
  } catch (err) {
    emit('error', 'Failed to setup MFA');
    console.error('Error setting up MFA:', err);
    // Reset the toggle if setup failed
    mfaEnabled.value = false;
    // Clear any partial state
    qrCodeUrl.value = '';
    mfaSecret.value = '';
    backupCodes.value = [];
    showSecret.value = false;
    secretCopied.value = false;
  } finally {
    loading.value = false;
  }
};

const verifyMFA = async () => {
  if (verificationCode.value.length !== 6) {
    emit('error', 'Please enter a valid 6-digit code');
    return;
  }

  if (!mfaSecret.value) {
    emit('error', 'MFA secret is missing. Please restart the setup process.');
    return;
  }

  loading.value = true;
  try {
    console.log('🔐 Verifying MFA with:', {
      token: verificationCode.value,
      secretLength: mfaSecret.value.length
    });

    let response;
    if (props.isLoginSetup) {
      // In login setup mode, use unauthenticated verification with credentials
      const context = sessionStorage.getItem('mfaLoginSetupContext');
      if (!context) {
        throw new Error('MFA setup context not found');
      }
      
      const { email, password } = JSON.parse(context);
      
      response = await fetch('/api/auth/mfa-setup-login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          email,
          password,
          token: verificationCode.value,
          secret: mfaSecret.value
        })
      });
    } else {
      // Normal authenticated verification
      response = await fetch('/api/auth/mfa/verify-setup', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${authStore.token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          token: verificationCode.value, // Backend expects 'token' field
          secret: mfaSecret.value // Backend expects 'secret' field
        })
      });
    }

    console.log('🔐 Verify setup response status:', response.status);

    if (response.ok) {
      const verifyData = await response.json();
      console.log('🔐 Verify setup successful:', verifyData);

      // Update backup codes if returned from verification
      if (verifyData.backup_codes && verifyData.backup_codes.length > 0) {
        backupCodes.value = verifyData.backup_codes;
      }

      // Now enable MFA after successful verification
      console.log('🔐 Enabling MFA...');
      
      let enableResponse;
      if (props.isLoginSetup) {
        // In login setup mode, use the special login-enable endpoint
        const context = sessionStorage.getItem('mfaLoginSetupContext');
        if (!context) {
          throw new Error('MFA setup context not found');
        }
        
        const { email, password } = JSON.parse(context);
        
        enableResponse = await fetch('/api/auth/mfa-enable-login', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            email,
            password,
            token: verificationCode.value,
            secret: mfaSecret.value,
            backup_codes: backupCodes.value
          })
        });
      } else {
        // Normal authenticated enable
        enableResponse = await fetch('/api/auth/mfa/enable', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${authStore.token}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            token: verificationCode.value,
            secret: mfaSecret.value,
            backup_codes: backupCodes.value
          })
        });
      }

      console.log('🔐 Enable MFA response status:', enableResponse.status);

      if (enableResponse.ok) {
        const enableData = await enableResponse.json();
        console.log('🔐 MFA enabled successfully:', enableData);
        
        mfaEnabled.value = true;
        mfaStep.value = 'enabled';
        verificationCode.value = '';
        emit('success', 'MFA enabled successfully! Your account is now more secure.');
      } else {
        const enableErrorData = await enableResponse.json().catch(() => ({}));
        console.error('🔐 Enable MFA failed:', enableErrorData);
        throw new Error(enableErrorData.message || 'Failed to enable MFA after verification');
      }
    } else {
      const errorData = await response.json().catch(() => ({}));
      console.error('🔐 Verify setup failed:', errorData);
      throw new Error(errorData.message || 'Invalid verification code');
    }
  } catch (err) {
    console.error('🔐 MFA verification error:', err);
    emit('error', err instanceof Error ? err.message : 'Invalid verification code. Please try again.');
  } finally {
    loading.value = false;
  }
};

const disableMFA = async () => {
  // Prompt for password confirmation
  const password = prompt('Please enter your password to disable MFA:');
  if (!password) {
    // User cancelled
    return;
  }

  loading.value = true;
  try {
    const response = await fetch('/api/auth/mfa/disable', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${authStore.token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        password: password
      })
    });

    if (response.ok) {
      mfaEnabled.value = false;
      mfaStep.value = 'setup';
      qrCodeUrl.value = '';
      verificationCode.value = '';
      backupCodes.value = [];
      mfaSecret.value = '';
      showSecret.value = false;
      secretCopied.value = false;
      emit('success', 'MFA disabled successfully');
    } else {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || 'Failed to disable MFA');
    }
  } catch (err) {
    emit('error', err instanceof Error ? err.message : 'Failed to disable MFA');
    console.error('Error disabling MFA:', err);
    // Reset the toggle if disable failed
    mfaEnabled.value = true;
  } finally {
    loading.value = false;
  }
};

const cancelMFASetup = () => {
  mfaStep.value = 'setup';
  qrCodeUrl.value = '';
  verificationCode.value = '';
  mfaSecret.value = '';
  backupCodes.value = [];
  showSecret.value = false;
  secretCopied.value = false;
};

// Format secret with spaces for better readability
const formatSecret = (secret: string) => {
  if (!secret) return '';
  // Add spaces every 4 characters for better readability
  return secret.replace(/(.{4})/g, '$1 ').trim();
};

// Copy secret to clipboard
const copySecret = async () => {
  if (!mfaSecret.value || secretCopied.value) return;
  
  try {
    await navigator.clipboard.writeText(mfaSecret.value);
    secretCopied.value = true;
    
    // Reset the button text after 2 seconds
    setTimeout(() => {
      secretCopied.value = false;
    }, 2000);
  } catch (err) {
    console.error('Failed to copy secret:', err);
    emit('error', 'Failed to copy to clipboard');
  }
};

// Expose methods for parent component access
defineExpose({
  startMFASetup
});
</script>

<template>
  <div class="bg-slate-800 rounded-xl border border-slate-700/50 hover:border-slate-600/50 transition-colors">
    <div class="px-4 py-3 bg-slate-700/30 border-b border-slate-700/50">
      <h2 class="text-lg font-medium text-white">Two-Factor Authentication</h2>
      <p class="text-sm text-slate-400 mt-1">Add an extra layer of security to your account</p>
    </div>
    
    <div class="p-6">
      <div class="flex flex-col gap-4">
        <!-- MFA Toggle / Status (hidden in login setup mode) -->
        <div v-if="!props.isLoginSetup" class="bg-slate-700/50 rounded-lg border border-slate-600/30 hover:border-slate-500/50 transition-colors p-4">
          <ToggleSwitch
            :modelValue="mfaEnabled"
            :disabled="loading"
            label="Enable Two-Factor Authentication"
            :description="mfaEnabled ? 'Your account is protected with 2FA' : 'Secure your account with an authenticator app'"
            @update:modelValue="toggleMFA"
          />
        </div>

        <!-- Setup Steps (hidden in login setup mode) -->
        <div v-if="showSetupSteps && !props.isLoginSetup" class="bg-slate-700/30 rounded-lg border border-slate-600/20 p-4">
          <h3 class="text-sm font-medium text-white mb-4">How to set up 2FA:</h3>
          <ol class="flex flex-col gap-3 text-sm text-slate-300">
            <li class="flex items-start gap-3">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5 flex-shrink-0">1</span>
              <span>Download an authenticator app like Google Authenticator or Authy</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5 flex-shrink-0">2</span>
              <span>Click the toggle above to start the setup process</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5 flex-shrink-0">3</span>
              <span>Scan the QR code with your authenticator app</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="bg-blue-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs font-medium mt-0.5 flex-shrink-0">4</span>
              <span>Enter the 6-digit code to complete setup</span>
            </li>
          </ol>
        </div>

              <!-- Verification Step -->
        <div v-if="showVerificationStep" class="bg-slate-700/30 rounded-lg border border-slate-600/20 p-6">
          <div class="flex flex-col gap-6">
            <!-- Header Section -->
            <div>
              <h3 class="text-sm font-medium text-white mb-2">Scan QR Code</h3>
              <p class="text-sm text-slate-400">Scan this QR code with your authenticator app:</p>
            </div>

            <!-- QR Code and Verification Grid -->
            <div class="grid grid-cols-1 lg:grid-cols-[auto_1fr] gap-8 items-start">
              <!-- QR Code Section -->
              <div class="flex justify-center lg:justify-start">
                <div class="bg-white p-4 rounded-lg">
                  <img 
                    v-if="qrCodeUrl && qrCodeUrl.startsWith('data:')"
                    :src="qrCodeUrl" 
                    alt="MFA QR Code" 
                    class="w-48 h-48 lg:w-44 lg:h-44"
                  />
                  <div 
                    v-else
                    class="w-48 h-48 lg:w-44 lg:h-44 bg-gray-200 flex items-center justify-center text-gray-500 text-xs"
                  >
                    {{ loading ? 'Loading QR Code...' : 'Generating QR Code...' }}
                  </div>
                </div>
              </div>

              <!-- Verification Components -->
              <div class="flex flex-col gap-6">
              <!-- Manual Secret Entry Option -->
              <div class="bg-slate-800/50 rounded-lg border border-slate-600/20 p-4">
                <button
                  @click="showSecret = !showSecret"
                  class="flex items-center gap-2 text-sm text-slate-400 hover:text-white transition-colors"
                >
                  <svg 
                    xmlns="http://www.w3.org/2000/svg" 
                    class="h-4 w-4 transition-transform"
                    :class="{ 'rotate-90': showSecret }"
                    fill="none" 
                    viewBox="0 0 24 24" 
                    stroke="currentColor"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                  Can't scan? Enter the code manually
                </button>
                
                <div v-if="showSecret" class="mt-4 flex flex-col gap-3">
                  <p class="text-sm text-slate-400">Enter this secret key in your authenticator app:</p>
                  <div class="bg-slate-700/50 rounded-lg p-3 border border-slate-600/30">
                    <div class="flex items-center justify-between gap-3">
                      <code class="text-sm font-mono text-green-400 select-all flex-1 break-all">{{ formatSecret(mfaSecret) }}</code>
                      <button
                        @click="copySecret"
                        :disabled="secretCopied"
                        class="px-3 py-1 text-xs rounded transition-all duration-200 flex-shrink-0"
                        :class="secretCopied 
                          ? 'bg-green-600 text-white cursor-default' 
                          : 'bg-slate-600 text-white hover:bg-slate-500 cursor-pointer'"
                        :title="secretCopied ? 'Copied to clipboard!' : 'Copy to clipboard'"
                      >
                        {{ secretCopied ? 'Copied!' : 'Copy' }}
                      </button>
                    </div>
                  </div>
                  <p class="text-xs text-slate-500">
                    Service: <span class="font-medium">Nosdesk</span> | 
                    Algorithm: <span class="font-medium">SHA1</span> | 
                    Digits: <span class="font-medium">6</span> | 
                    Period: <span class="font-medium">30s</span>
                  </p>
                </div>
              </div>
              
              <!-- Verification Input Section -->
              <div class="flex flex-col gap-4">
                <div>
                  <h4 class="text-sm font-medium text-white mb-2">Enter Verification Code</h4>
                  <p class="text-sm text-slate-400">Enter the 6-digit code from your authenticator app:</p>
                </div>
                
                <div class="flex gap-3">
                  <div class="bg-slate-700/50 rounded-lg border border-slate-600/30 hover:border-slate-500/50 transition-colors flex-1">
                    <input
                      v-model="verificationCode"
                      type="text"
                      maxlength="6"
                      class="w-full px-4 py-3 bg-transparent text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none text-center tracking-widest"
                      placeholder="000000"
                    />
                  </div>
                  <button
                    @click="verifyMFA"
                    :disabled="verificationCode.length !== 6 || loading"
                    class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center transition-colors"
                  >
                                      <span v-if="loading" class="animate-spin h-4 w-4 mr-2">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 0 1 8-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 0 1 4 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  </span>
                    Verify
                  </button>
                </div>
                
                <button
                  @click="cancelMFASetup"
                  class="self-start text-sm text-slate-400 hover:text-white transition-colors"
                >
                  Cancel setup
                </button>
              </div>
            </div>
          </div>
        </div>
        </div>

        <!-- Backup Codes Display -->
        <div v-if="backupCodes.length > 0 && mfaStep === 'verify'" class="bg-amber-600/10 border border-amber-600/20 rounded-lg p-4">
          <div class="flex flex-col gap-4">
            <div>
              <h4 class="text-sm font-medium text-amber-400 mb-2">Backup Codes</h4>
              <p class="text-sm text-slate-300">Save these backup codes in a secure location. You can use them to access your account if you lose your authenticator device:</p>
            </div>
            <div class="bg-slate-700/50 rounded-lg p-4 font-mono text-sm text-white">
              <div class="grid grid-cols-2 gap-2">
                <div v-for="code in backupCodes" :key="code" class="text-center p-2 bg-slate-800/50 rounded">{{ code }}</div>
              </div>
            </div>
            <p class="text-xs text-amber-400 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              Each backup code can only be used once.
            </p>
          </div>
        </div>

        <!-- Enabled State -->
        <div v-if="showEnabledState" class="bg-green-600/10 border border-green-600/20 rounded-lg p-4">
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-1">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-green-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <h3 class="text-sm font-medium text-green-400">2FA is enabled</h3>
            </div>
            <p class="text-sm text-slate-300">Your account is protected with two-factor authentication. You'll need to enter a code from your authenticator app when signing in.</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template> 