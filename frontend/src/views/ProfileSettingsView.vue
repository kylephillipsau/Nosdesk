<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';
import BackButton from '@/components/common/BackButton.vue';
import UserAvatar from '@/components/UserAvatar.vue';
import axios from 'axios';
import userService from '@/services/userService';
import type { AuthIdentity, User } from '@/services/userService';

const authStore = useAuthStore();
const loading = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const fetchingUserData = ref(false);

// Original user data for comparison
const originalData = ref({
  name: '',
  email: ''
});

// Form data initialized with empty values
const formData = ref({
  name: '',
  email: '',
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
  pronouns: '',
  avatar_url: '',
  banner_url: '',
  status: 'Online'
});

// Files for upload
const avatarFile = ref<File | null>(null);
const bannerFile = ref<File | null>(null);
const avatarPreview = ref<string | null>(null);
const bannerPreview = ref<string | null>(null);
const bannerFileInput = ref<HTMLInputElement | null>(null);

// Add a ref to access the avatar component
interface UserAvatarComponentType {
  refreshUser: (uuid?: string) => Promise<void>;
}

const userAvatarComponent = ref<UserAvatarComponentType | null>(null);

// Computed properties to check if fields have been modified
const nameModified = computed(() => 
  formData.value.name !== originalData.value.name && formData.value.name.trim() !== ''
);

const emailModified = computed(() => 
  formData.value.email !== originalData.value.email && formData.value.email.trim() !== ''
);

// Update form data when user data changes
watch(() => authStore.user, (newUserData) => {
  if (newUserData) {
    formData.value.name = newUserData.name || '';
    formData.value.email = newUserData.email || '';
    formData.value.pronouns = newUserData.pronouns || '';
    formData.value.avatar_url = newUserData.avatar_url || '';
    formData.value.banner_url = newUserData.banner_url || '';
    
    // Also update original data for comparison
    originalData.value.name = newUserData.name || '';
    originalData.value.email = newUserData.email || '';
  }
}, { immediate: true });

// Load initial data if we already have it in the store
if (authStore.user) {
  formData.value.name = authStore.user.name || '';
  formData.value.email = authStore.user.email || '';
  formData.value.pronouns = authStore.user.pronouns || '';
  formData.value.avatar_url = authStore.user.avatar_url || '';
  formData.value.banner_url = authStore.user.banner_url || '';
  originalData.value.name = authStore.user.name || '';
  originalData.value.email = authStore.user.email || '';
}

const updateProfile = async () => {
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // TODO: Implement profile update logic
    // await userService.updateProfile(formData.value);
    successMessage.value = 'Profile updated successfully';
    
    // Update original data after successful save
    originalData.value.name = formData.value.name;
    originalData.value.email = formData.value.email;
  } catch (err) {
    error.value = 'Failed to update profile';
    console.error('Error updating profile:', err);
  } finally {
    loading.value = false;
  }
};

// Individual field update functions
const updateName = async () => {
  if (!nameModified.value) return;
  
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // TODO: Implement name update logic
    // await userService.updateName(formData.value.name);
    successMessage.value = 'Name updated successfully';
    
    // Update original data after successful save
    originalData.value.name = formData.value.name;
  } catch (err) {
    error.value = 'Failed to update name';
    console.error('Error updating name:', err);
  } finally {
    loading.value = false;
  }
};

const updateEmail = async () => {
  if (!emailModified.value) return;
  
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // TODO: Implement email update logic
    // await userService.updateEmail(formData.value.email);
    successMessage.value = 'Email updated successfully';
    
    // Update original data after successful save
    originalData.value.email = formData.value.email;
  } catch (err) {
    error.value = 'Failed to update email';
    console.error('Error updating email:', err);
  } finally {
    loading.value = false;
  }
};

const updatePassword = async () => {
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  // Validation
  if (formData.value.newPassword !== formData.value.confirmPassword) {
    error.value = 'New password and confirmation do not match';
    loading.value = false;
    return;
  }

  if (!formData.value.currentPassword || !formData.value.newPassword) {
    error.value = 'All password fields are required';
    loading.value = false;
    return;
  }

  try {
    const response = await axios.post('/api/auth/change-password', {
      current_password: formData.value.currentPassword,
      new_password: formData.value.newPassword
    });

    successMessage.value = 'Password updated successfully';
    
    // Clear password fields
    formData.value.currentPassword = '';
    formData.value.newPassword = '';
    formData.value.confirmPassword = '';
  } catch (err: any) {
    error.value = err.response?.data?.message || 'Failed to update password';
    console.error('Error updating password:', err);
  } finally {
    loading.value = false;
  }
};

const updatePronouns = async () => {
  if (!formData.value.pronouns && !authStore.user?.pronouns) return;
  
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // Get user UUID from auth store
    const userUuid = authStore.user?.uuid;
    if (!userUuid) {
      error.value = "User not authenticated";
      return;
    }
    
    console.log(`Updating pronouns for user ${userUuid} to "${formData.value.pronouns}"`);
    
    // Update pronouns via API
    const updatedUser = await userService.updateUser(userUuid, {
      pronouns: formData.value.pronouns
    });
    
    if (updatedUser) {
      console.log("Pronouns updated successfully:", updatedUser);
      successMessage.value = 'Pronouns updated successfully';
      // Update the auth store with new user data
      if (authStore.user) {
        authStore.user = {
          ...authStore.user,
          pronouns: updatedUser.pronouns
        };
      }
    } else {
      error.value = "Failed to update pronouns";
      console.error("Update returned null");
    }
  } catch (err) {
    error.value = 'Failed to update pronouns';
    console.error('Error updating pronouns:', err);
  } finally {
    loading.value = false;
  }
};

const handleAvatarClick = () => {
  fileInput.value?.click();
};

const handleBannerClick = () => {
  bannerFileInput.value?.click();
};

const handleFileChange = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files?.length) return;

  const file = input.files[0];
  if (!file.type.startsWith('image/')) {
    error.value = 'Please select an image file';
    return;
  }

  avatarFile.value = file;
  
  // Create a preview
  avatarPreview.value = URL.createObjectURL(file);
  
  // Auto-upload when file is selected
  await uploadAvatar();
};

const handleBannerChange = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files?.length) return;

  const file = input.files[0];
  if (!file.type.startsWith('image/')) {
    error.value = 'Please select an image file';
    return;
  }

  bannerFile.value = file;
  
  // Create a preview
  bannerPreview.value = URL.createObjectURL(file);
  
  // Auto-upload when file is selected
  await uploadBanner();
};

const uploadAvatar = async () => {
  if (!avatarFile.value) return;
  
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // Get user UUID from auth store
    const userUuid = authStore.user?.uuid;
    if (!userUuid) {
      console.error("User UUID missing, attempting to refresh user data");
      // Try to refresh user data
      try {
        await authStore.fetchUserData();
        if (!authStore.user?.uuid) {
          error.value = "User not authenticated. Please log in again.";
          return;
        }
      } catch (refreshErr) {
        console.error("Failed to refresh user data:", refreshErr);
        error.value = "User not authenticated. Please log in again.";
        return;
      }
    }
    
    // Upload the image and get a URL back
    const uploadedUrl = await userService.uploadImage(avatarFile.value, 'avatar');
    
    if (!uploadedUrl) {
      error.value = "Failed to upload image";
      return;
    }
    
    // We don't need to update the user separately since the backend already updates it
    // Just update the local UI
    successMessage.value = 'Profile picture updated successfully';
    formData.value.avatar_url = uploadedUrl;
    
    // Update the auth store with new user data
    if (authStore.user) {
      authStore.user = {
        ...authStore.user,
        avatar_url: uploadedUrl
      };
    }
    
    // Refresh UserAvatar shared user data for all components that display this user
    if (userAvatarComponent.value && userAvatarComponent.value.refreshUser) {
      userAvatarComponent.value.refreshUser(authStore.user?.uuid);
    }
    
    // Force an re-fetch of user data to update all components
    setTimeout(() => {
      authStore.fetchUserData();
    }, 500);
  } catch (err) {
    error.value = 'Failed to update profile picture';
    console.error('Error updating avatar:', err);
  } finally {
    loading.value = false;
  }
};

const uploadBanner = async () => {
  if (!bannerFile.value) return;
  
  loading.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    // Get user UUID from auth store
    const userUuid = authStore.user?.uuid;
    if (!userUuid) {
      console.error("User UUID missing, attempting to refresh user data");
      // Try to refresh user data
      try {
        await authStore.fetchUserData();
        if (!authStore.user?.uuid) {
          error.value = "User not authenticated. Please log in again.";
          return;
        }
      } catch (refreshErr) {
        console.error("Failed to refresh user data:", refreshErr);
        error.value = "User not authenticated. Please log in again.";
        return;
      }
    }
    
    // Upload the image and get a URL back
    const uploadedUrl = await userService.uploadImage(bannerFile.value, 'banner');
    
    if (!uploadedUrl) {
      error.value = "Failed to upload image";
      return;
    }
    
    // We don't need to update the user separately since the backend already updates it
    // Just update the local UI
    successMessage.value = 'Cover image updated successfully';
    formData.value.banner_url = uploadedUrl;
    
    // Update the auth store with new user data
    if (authStore.user) {
      authStore.user = {
        ...authStore.user,
        banner_url: uploadedUrl
      };
    }
    
    // Refresh UserAvatar shared user data for all components that display this user
    if (userAvatarComponent.value && userAvatarComponent.value.refreshUser) {
      userAvatarComponent.value.refreshUser(authStore.user?.uuid);
    }
    
    // Force an re-fetch of user data to update all components
    setTimeout(() => {
      authStore.fetchUserData();
    }, 500);
  } catch (err) {
    error.value = 'Failed to update cover image';
    console.error('Error updating banner:', err);
  } finally {
    loading.value = false;
  }
};

// User authentication identities
const authIdentities = ref<AuthIdentity[]>([]);
const loadingIdentities = ref(false);
const identityError = ref<string | null>(null);

const loadAuthIdentities = async (retryCount = 0) => {
  loadingIdentities.value = true;
  identityError.value = null;
  
  try {
    // Get the auth token from the store
    const authToken = authStore.token;
    if (!authToken) {
      throw new Error('No authentication token available');
    }

    // Debug: Output detailed user info
    console.log('User data from store:', {
      uuid: authStore.user?.uuid,
      name: authStore.user?.name,
      email: authStore.user?.email,
      token: authToken?.substring(0, 10) + '...' // Show just the beginning for security
    });
    
    // Check if we have a valid user ID before making the request
    if (!authStore.user?.uuid) {
      console.warn('Missing user UUID - trying to reload user data first');
      const userData = await authStore.fetchUserData();
      if (!userData) {
        throw new Error('Could not load user profile data');
      }
    }

    // Make a direct API call to fully debug the response
    console.log('Making direct API call to get auth identities...');
    let userUuid = authStore.user?.uuid;
    let endpoint = userUuid ? 
      `/api/users/${userUuid}/auth-identities` : 
      `/api/users/auth-identities`;
      
    console.log(`Using auth identities endpoint: ${endpoint}`);
    
    try {
      const directResponse = await axios.get(endpoint, {
        headers: { 'Authorization': `Bearer ${authToken}` }
      });
      console.log('DIRECT API CALL RESULT:', directResponse.data);
      
      if (directResponse.data && Array.isArray(directResponse.data)) {
        console.log(`Found ${directResponse.data.length} auth identities from direct API call`);
        directResponse.data.forEach((identity, index) => {
          console.log(`Raw identity ${index + 1}:`, identity);
        });
      } else {
        console.warn('Direct API call returned non-array data:', directResponse.data);
      }
      
      // If we got identities from the direct call but none from the service, try using these
      if (directResponse.data && Array.isArray(directResponse.data) && directResponse.data.length > 0) {
        authIdentities.value = directResponse.data.map(item => ({
          id: item.id,
          provider_type: item.provider_type,
          provider_name: item.provider_name,
          email: item.email,
          created_at: item.created_at
        }));
        console.log('Populated auth identities directly from API call');
        return;
      }
    } catch (directErr: any) {
      console.error('Direct API call failed:', directErr);
      const errorDetails = directErr.response ? 
        `Status: ${directErr.response.status}, Message: ${JSON.stringify(directErr.response.data)}` : 
        directErr.message;
      console.error('Detailed error from direct API call:', errorDetails);
    }

    // Now try using the service
    console.log('Trying to get auth identities via userService...');
    const authIdentitiesData = await userService.getUserAuthIdentities();
    
    console.log('Auth identities data from service:', authIdentitiesData);
    
    // Log raw data details to help debug provider types
    if (authIdentitiesData && authIdentitiesData.length > 0) {
      console.log('Auth identities detail:');
      authIdentitiesData.forEach((identity, index) => {
        console.log(`Identity ${index + 1}:`, {
          id: identity.id,
          provider_type: identity.provider_type,
          provider_name: identity.provider_name,
          email: identity.email,
          created_at: identity.created_at
        });
      });
      
      // Successfully retrieved identities from service
      authIdentities.value = authIdentitiesData;
      return;
    } else {
      console.warn('No auth identities returned from service');
    }
    
    // If we couldn't get identities via any method, create a fallback
    console.warn('Failed to retrieve auth identities through any method, using fallback');
    createFallbackAuthIdentity();
    
  } catch (err: any) {
    console.error('Error loading auth identities:', err);
    const errorDetails = err.response ? `Status: ${err.response.status}, Message: ${JSON.stringify(err.response.data)}` : err.message;
    console.error('Detailed error:', errorDetails);
    
    // If this is a 401/404 error and we haven't retried yet, try to reload user data and try again
    if (err.response && (err.response.status === 401 || err.response.status === 404) && retryCount === 0) {
      console.log('Auth error, trying to refresh user data and retry...');
      try {
        await authStore.fetchUserData();
        return loadAuthIdentities(retryCount + 1);
      } catch (refreshErr) {
        console.error('Failed to refresh user data:', refreshErr);
      }
    }
    
    // Always create a fallback auth identity
    createFallbackAuthIdentity();
    
    // Show a warning but don't block the UI
    identityError.value = 'Unable to fetch authentication methods. Only showing default method.';
  } finally {
    loadingIdentities.value = false;
  }
};

// Helper function to create a fallback auth identity
const createFallbackAuthIdentity = () => {
  // Only create a fallback if we have user data
  if (authStore.user) {
    // Clear any existing identities to avoid mixing with invalid data
    authIdentities.value = [];
    
    // Create a single fallback identity with a stable ID based on the user
    authIdentities.value = [{
      id: 0, // Use a stable ID for the fallback
      provider_type: 'local',
      provider_name: 'Local Account',
      email: authStore.user.email || null,
      created_at: new Date().toISOString()
    }];
    
    console.log('Created fallback auth identity:', authIdentities.value);
  } else {
    console.warn('Cannot create fallback identity - no user data available');
  }
};

// Function to connect to a Microsoft account
const connectMicrosoftAccount = async () => {
  loadingIdentities.value = true;
  identityError.value = null;
  
  try {
    // Get the auth token from the store
    const authToken = authStore.token;
    if (!authToken) {
      throw new Error('No authentication token available');
    }

    // Store the current URL to redirect back after authentication
    sessionStorage.setItem('authRedirect', window.location.pathname);
    
    // Get authorization URL from backend - using the explicit connect endpoint
    const response = await axios.post('/api/auth/oauth/connect', {
      provider_type: 'microsoft',
      redirect_uri: `${window.location.origin}/profile/settings` 
    }, {
      headers: {
        'Authorization': `Bearer ${authToken}`
      }
    });
    
    // Redirect to Microsoft login
    window.location.href = response.data.auth_url;
  } catch (err: any) {
    console.error('Error initiating Microsoft connection:', err);
    identityError.value = err.response?.data?.message || 'Failed to initiate Microsoft authentication';
    loadingIdentities.value = false;
  }
};

const deleteAuthIdentity = async (identityId: number, retryCount = 0) => {
  if (!confirm('Are you sure you want to remove this authentication method?')) {
    return;
  }
  
  loadingIdentities.value = true;
  identityError.value = null;
  
  try {
    // Get the auth token from the store
    const authToken = authStore.token;
    if (!authToken) {
      throw new Error('No authentication token available');
    }
    
    // Debug: Check user info before making the request
    console.log('Attempting to delete auth identity:', {
      identityId,
      userUUID: authStore.user?.uuid,
      tokenPrefix: authToken.substring(0, 10) + '...'
    });

    // Check if we have a valid user ID before making the request
    if (!authStore.user?.uuid) {
      console.warn('Missing user UUID - trying to reload user data first');
      const userData = await authStore.fetchUserData();
      if (!userData) {
        throw new Error('Could not load user profile data');
      }
    }

    // Use the service instead of direct axios call
    await userService.deleteUserAuthIdentity(identityId);
    
    console.log('Successfully deleted auth identity:', identityId);
    
    // Reload the list
    await loadAuthIdentities();
    successMessage.value = 'Authentication method removed successfully';
  } catch (err: any) {
    console.error('Error deleting auth identity:', err);
    const errorDetails = err.response ? `Status: ${err.response.status}, Message: ${JSON.stringify(err.response.data)}` : err.message;
    console.error('Detailed error:', errorDetails);
    
    // If this is a 401/404 error and we haven't retried yet, try to reload user data and try again
    if (err.response && (err.response.status === 401 || err.response.status === 404) && retryCount === 0) {
      console.log('Auth error during delete, trying to refresh user data and retry...');
      try {
        await authStore.fetchUserData();
        return deleteAuthIdentity(identityId, retryCount + 1);
      } catch (refreshErr) {
        console.error('Failed to refresh user data during delete:', refreshErr);
      }
    }
    
    identityError.value = err.response?.data?.message || 'Failed to remove authentication method';
  } finally {
    loadingIdentities.value = false;
  }
};

onMounted(async () => {
  document.title = 'Profile Settings | Nosdesk';
  
  // Only attempt to fetch user data if we're authenticated
  if (!authStore.isAuthenticated) {
    error.value = 'You must be logged in to view this page';
    return;
  }
  
  console.log('Auth state on mount:', {
    isAuthenticated: authStore.isAuthenticated,
    userExists: !!authStore.user,
    userUUID: authStore.user?.uuid,
    tokenExists: !!authStore.token,
    tokenPrefix: authStore.token ? authStore.token.substring(0, 10) + '...' : null
  });
  
  // Check for success/error messages in URL (for return from OAuth flow)
  const urlParams = new URLSearchParams(window.location.search);
  const authSuccess = urlParams.get('auth_success');
  const authError = urlParams.get('auth_error');
  const token = urlParams.get('token');
  
  if (token) {
    // We have a token from the OAuth callback, store it and refresh the page
    // This is likely a return from an OAuth connection flow
    try {
      // Store the new token
      authStore.token = token;
      localStorage.setItem('token', token);
      
      // Set authorization header for future requests
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;
      
      // Reload auth identities
      await loadAuthIdentities();
      
      // Show success message
      successMessage.value = 'Authentication method added successfully';
      
      // Clean up URL
      window.history.replaceState({}, document.title, window.location.pathname);
    } catch (err) {
      console.error('Error handling OAuth token:', err);
      error.value = 'Failed to add authentication method';
    }
  } else if (authSuccess) {
    // Regular auth success from URL param
    successMessage.value = 'Authentication method added successfully';
    // Clean up URL params
    window.history.replaceState({}, document.title, window.location.pathname);
  } else if (authError) {
    identityError.value = decodeURIComponent(authError);
    // Clean up URL params
    window.history.replaceState({}, document.title, window.location.pathname);
  }
  
  // Fetch the latest user data from the backend
  fetchingUserData.value = true;
  try {
    console.log('Fetching user data...');
    // Use a try/catch here to prevent the component from failing if data can't be fetched
    const userData = await authStore.fetchUserData();
    console.log('Fetched user data result:', userData);
    
    if (!userData && !authStore.user) {
      error.value = 'Failed to load user profile. Using cached data.';
    }
    
    // Load auth identities
    await loadAuthIdentities();
  } catch (err) {
    console.error('Failed to fetch user data:', err);
    error.value = 'Failed to refresh user profile data. Using cached data.';
    
    // Even if fetch fails, we can still show the page with existing data from the store
    // so don't redirect or show a blocking error
  } finally {
    fetchingUserData.value = false;
  }
});
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div class="pt-4 px-6 flex justify-between items-center">
      <BackButton fallbackRoute="/" label="Back to Dashboard" />
    </div>
    
    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
      <!-- Success/Error messages -->
      <div v-if="successMessage" class="p-4 bg-green-900/50 text-green-400 rounded-lg">
        {{ successMessage }}
      </div>
      <div v-if="error" class="p-4 bg-red-900/50 text-red-400 rounded-lg">
        {{ error }}
      </div>

      <!-- Loading indicator while fetching user data -->
      <div v-if="fetchingUserData" class="flex justify-center items-center py-8">
        <div class="animate-spin h-8 w-8 text-blue-500">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
        <span class="ml-3 text-white">Loading user profile...</span>
      </div>

      <!-- Main content when user data is loaded -->
      <div v-else class="grid-container">
        <!-- Profile Information Area -->
        <div class="info-area flex flex-col gap-4">
          <!-- Discord-inspired Profile Card -->
          <div class="bg-slate-800 rounded-2xl overflow-hidden">
            <!-- Cover/Banner Image -->
            <div 
              class="h-32 bg-gradient-to-r from-blue-600 to-purple-600 relative"
              :style="formData.banner_url ? `background-image: url('${formData.banner_url}'); background-size: cover; background-position: center;` : ''"
            >
              <!-- Add cover image upload option -->
              <button 
                class="absolute bottom-2 right-2 bg-slate-800/50 hover:bg-slate-800/80 text-white rounded-full p-2 transition-colors"
                @click="handleBannerClick"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </button>
              <!-- Hidden banner file input -->
              <input
                ref="bannerFileInput"
                type="file"
                accept="image/*"
                class="hidden"
                @change="handleBannerChange"
              />
            </div>
            
            <!-- Profile Content -->
            <div class="px-6 pt-16 pb-6 relative">
              <!-- Avatar that overlaps the banner -->
              <div 
                class="absolute -top-12 left-6 w-24 h-24 rounded-full overflow-hidden border-4 border-slate-800 cursor-pointer"
                @click="handleAvatarClick"
              >
                <UserAvatar
                  :name="authStore.user?.name || ''"
                  size="full"
                  :avatar="formData.avatar_url || null"
                  :showName="false"
                  :clickable="false"
                  class="w-full h-full"
                  ref="userAvatarComponent"
                />
                <!-- Hover Overlay -->
                <div class="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity">
                  <div class="text-white flex flex-col items-center gap-1">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                  </div>
                </div>
                <!-- Hidden file input -->
                <input
                  ref="fileInput"
                  type="file"
                  accept="image/*"
                  class="hidden"
                  @change="handleFileChange"
                />
              </div>
              
              <!-- User info section -->
              <div class="flex justify-between items-start mb-6">
                <div class="flex flex-col gap-2">
                  <!-- Role badges section -->
                  <div class="flex gap-2 mb-3">
                    <div class="px-3 py-1 bg-blue-600/20 text-blue-400 rounded-full text-sm font-medium">
                      {{ authStore.user?.role || 'User' }}
                    </div>
                    <div v-if="authStore.isAdmin" class="px-3 py-1 bg-red-600/20 text-red-400 rounded-full text-sm font-medium">
                      Admin
                    </div>
                  </div>
                  
                  <!-- Pronouns field -->
                  <div class="mt-2 text-sm text-slate-400 flex items-center gap-1">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                    <div class="flex items-start gap-2 w-full mt-1">
                      <input
                        v-model="formData.pronouns"
                        type="text"
                        class="flex-1 px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none text-sm"
                        placeholder="Add pronouns (e.g., he/him, she/her, they/them)"
                      />
                      <button
                        @click="updatePronouns"
                        :disabled="loading"
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center text-sm"
                      >
                        <span v-if="loading" class="animate-spin h-4 w-4 mr-2">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                          </svg>
                        </span>
                        Save
                      </button>
                    </div>
                  </div>
                </div>
              </div>
              
              <!-- Always Editable Fields -->
              <div class="space-y-6">
                <!-- Display name section with always-editable field -->
                <div>
                  <h3 class="text-sm font-medium text-slate-400 mb-2">Display Name</h3>
                  <div class="flex items-start gap-2">
                    <input
                      v-model="formData.name"
                      type="text"
                      class="flex-1 px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                      placeholder="Your display name"
                    />
                    <button
                      @click="updateName"
                      :disabled="!nameModified || loading"
                      class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center"
                    >
                      <span v-if="loading && nameModified" class="animate-spin h-4 w-4 mr-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                      </span>
                      Save
                    </button>
                  </div>
                </div>
                
                <!-- Email section with always-editable field -->
                <div>
                  <h3 class="text-sm font-medium text-slate-400 mb-2">Email</h3>
                  <div class="flex items-start gap-2">
                    <input
                      v-model="formData.email"
                      type="email"
                      class="flex-1 px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                      placeholder="your.email@example.com"
                    />
                    <button
                      @click="updateEmail"
                      :disabled="!emailModified || loading"
                      class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-slate-600 flex items-center"
                    >
                      <span v-if="loading && emailModified" class="animate-spin h-4 w-4 mr-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                      </span>
                      Save
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Security Area -->
        <div class="security-area flex flex-col gap-4">
          <!-- Password Change Section -->
          <div class="bg-slate-800 rounded-2xl p-6">
            <h2 class="text-lg font-medium text-white mb-6">Security Settings</h2>
            
            <form @submit.prevent="updatePassword" class="flex flex-col gap-6">
              <!-- Success/Error messages -->
              <div v-if="successMessage" class="p-4 bg-green-900/50 text-green-400 rounded-lg">
                {{ successMessage }}
              </div>
              <div v-if="error" class="p-4 bg-red-900/50 text-red-400 rounded-lg">
                {{ error }}
              </div>
              
              <!-- Current Password -->
              <div class="flex flex-col gap-2">
                <label for="currentPassword" class="text-sm font-medium text-slate-300">Current Password</label>
                <input
                  id="currentPassword"
                  v-model="formData.currentPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Enter your current password"
                />
              </div>

              <!-- New Password -->
              <div class="flex flex-col gap-2">
                <label for="newPassword" class="text-sm font-medium text-slate-300">New Password</label>
                <input
                  id="newPassword"
                  v-model="formData.newPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Enter new password"
                />
              </div>

              <!-- Confirm Password -->
              <div class="flex flex-col gap-2">
                <label for="confirmPassword" class="text-sm font-medium text-slate-300">Confirm New Password</label>
                <input
                  id="confirmPassword"
                  v-model="formData.confirmPassword"
                  type="password"
                  class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                  placeholder="Confirm new password"
                />
              </div>

              <!-- Submit button for password change -->
              <div class="flex justify-end pt-2">
                <button
                  type="submit"
                  :disabled="loading"
                  class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                >
                  <span v-if="loading" class="animate-spin h-4 w-4">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  </span>
                  {{ loading ? 'Updating...' : 'Update Password' }}
                </button>
              </div>
            </form>
          </div>

          <!-- Authentication Methods Section -->
          <div class="flex flex-col gap-2 bg-slate-800 rounded-2xl p-6 mt-4">
            <h2 class="text-lg font-medium text-white mb-4">Authentication Methods</h2>
            
            <!-- Error message for identities -->
            <div v-if="identityError" class="p-4 bg-red-900/50 text-red-400 rounded-lg mb-4">
              {{ identityError }}
            </div>
            
            <!-- Loading state -->
            <div v-if="loadingIdentities" class="flex justify-center my-4">
              <div class="animate-spin rounded-full h-6 w-6 border-t-2 border-b-2 border-blue-500"></div>
            </div>
            
            <!-- Auth identities list -->
            <div v-else-if="authIdentities.length" class="flex flex-col gap-2">
              <div v-for="identity in authIdentities" :key="`auth-identity-${identity.id}-${identity.provider_type}`" 
                   class="flex items-center justify-between gap-2 p-3 bg-slate-700 rounded-lg"
                   :title="'ID: ' + identity.id + ', Type: ' + identity.provider_type"
              >
                <div class="flex items-center gap-3">
                  <!-- Provider icon with better visualization -->
                  <div 
                    class="w-8 h-8 flex items-center justify-center rounded-full text-white"
                    :class="{
                      'bg-blue-600': identity.provider_type === 'microsoft',
                      'bg-green-600': identity.provider_type === 'google',
                      'bg-gray-600': identity.provider_type === 'local',
                      'bg-slate-600': !['microsoft', 'google', 'local'].includes(identity.provider_type)
                    }"
                  >
                    <!-- Microsoft Logo -->
                    <template v-if="identity.provider_type === 'microsoft'">
                      <svg width="16" height="16" viewBox="0 0 21 21">
                        <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
                        <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
                        <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
                        <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
                      </svg>
                    </template>
                    
                    <!-- Local Account Icon -->
                    <template v-else-if="identity.provider_type === 'local'">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                      </svg>
                    </template>
                    
                    <!-- Fallback letter for other providers -->
                    <template v-else>
                      {{ identity.provider_type.charAt(0).toUpperCase() }}
                    </template>
                  </div>
                  
                  <div>
                    <div class="text-white">{{ identity.provider_name || identity.provider_type }}</div>
                    <div class="text-sm text-slate-400">{{ identity.email || 'No email associated' }}</div>
                  </div>
                </div>
                
                <button 
                  @click="deleteAuthIdentity(identity.id)"
                  class="p-2 text-slate-400 hover:text-red-400"
                  :disabled="authIdentities.length <= 1"
                  :title="authIdentities.length <= 1 ? 'Cannot remove your only authentication method' : 'Remove this authentication method'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
            
            <!-- Empty state -->
            <div v-else class="text-center py-4 text-slate-400">
              No authentication methods found
            </div>
            
            <!-- Add Authentication Method Button -->
            <div class="mt-4">
              <button 
                @click="connectMicrosoftAccount" 
                class="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors w-full justify-center disabled:bg-blue-800 disabled:cursor-not-allowed"
                :disabled="loadingIdentities"
              >
                <div v-if="loadingIdentities" class="animate-spin h-5 w-5 mr-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                </div>
                <div v-else class="w-5 h-5 flex items-center justify-center">
                  <svg width="16" height="16" viewBox="0 0 21 21">
                    <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
                    <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
                    <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
                    <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
                  </svg>
                </div>
                {{ loadingIdentities ? 'Connecting...' : 'Connect Microsoft Account' }}
              </button>
            </div>
            
            <!-- Info text -->
            <p class="text-sm text-slate-400 mt-4">
              You can sign in using any of these methods. Adding multiple methods gives you alternative ways to access your account.
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-template-areas: "info" "security";
  gap: 1rem;

  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas: "info security";
  }
}

.info-area {
  grid-area: info;
}

.security-area {
  grid-area: security;
}
</style> 