<!-- Microsoft Configuration Guide -->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import axios from "axios";
import MicrosoftGraphService from "@/services/MicrosoftGraphService";
import BackButton from "@/components/common/BackButton.vue";

const props = defineProps<{
  providerId?: number;
  mode?: string;
  showBackButton?: boolean;
  isEmbedded?: boolean;
}>();

const emit = defineEmits(["configured"]);

const route = useRoute();
const router = useRouter();

// Get providerId from route params if not provided via props
const routeProviderId = computed(() => {
  return (
    props.providerId ||
    (route.params.providerId ? Number(route.params.providerId) : undefined)
  );
});

// Get mode from query params if not provided via props
const configMode = computed(() => {
  return props.mode || route.query.mode?.toString() || "auth";
});

// Determine back button route based on mode
const backRoute = computed(() => {
  if (configMode.value === "graph") {
    return "/admin/data-import/microsoft-graph";
  }
  return "/admin/auth-providers";
});

const isLoading = ref(false);
const errorMessage = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const configurationStatus = ref<any>(null);
const graphApiResults = ref<any>(null);
const permissionTestResults = ref<any>(null);

// Test configuration
const testConfiguration = async () => {
  if (!routeProviderId.value) {
    errorMessage.value = "No provider ID available for testing";
    return;
  }

  isLoading.value = true;
  errorMessage.value = null;
  successMessage.value = null;
  configurationStatus.value = null;

  try {
    const response = await axios.get(
      `/api/auth/providers/${routeProviderId.value}/test`
    );

    if (response.data.status === "success") {
      successMessage.value = response.data.message;
      configurationStatus.value = response.data.details;
    } else {
      errorMessage.value = response.data.message;
      if (response.data.details) {
        errorMessage.value += `\n\nDetails: ${response.data.details}`;
      }
    }
  } catch (error: any) {
    console.error("Error testing Microsoft configuration:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to test configuration";
  } finally {
    isLoading.value = false;
  }
};

// Test all Graph API permissions comprehensively
const testAllPermissions = async () => {
  if (!routeProviderId.value) {
    errorMessage.value = "No provider ID available for testing";
    return;
  }

  isLoading.value = true;
  errorMessage.value = null;
  successMessage.value = null;
  permissionTestResults.value = null;
  graphApiResults.value = null;

  try {
    const results = await MicrosoftGraphService.testAllPermissions(
      routeProviderId.value
    );

    permissionTestResults.value = results;
    
    // Set overall success message
    const successCount = Object.values(results).filter((r: any) => r.status === 'success').length;
    const totalCount = Object.keys(results).length;
    
    if (successCount === totalCount) {
      successMessage.value = `All ${totalCount} permissions are working correctly!`;
    } else {
      successMessage.value = `${successCount} out of ${totalCount} permissions are working. Check details below.`;
    }
  } catch (error: any) {
    console.error("Error testing Graph API permissions:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to test Microsoft Graph API permissions";

    if (error.response?.data?.data?.error) {
      errorMessage.value += `\n\nGraph API Error: ${JSON.stringify(
        error.response.data.data.error
      )}`;
    }
  } finally {
    isLoading.value = false;
  }
};

// Legacy single test for backwards compatibility
const testGraphApi = async () => {
  if (!routeProviderId.value) {
    errorMessage.value = "No provider ID available for testing";
    return;
  }

  isLoading.value = true;
  errorMessage.value = null;
  successMessage.value = null;
  graphApiResults.value = null;

  try {
    const response = await MicrosoftGraphService.getUsers(
      undefined,
      "displayName,givenName,surname,userPrincipalName,id,mail,jobTitle,department",
      routeProviderId.value
    );

    graphApiResults.value = response.data;
    successMessage.value = "Graph API call successful";
  } catch (error: any) {
    console.error("Error testing Graph API:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to call Microsoft Graph API";

    if (error.response?.data?.data?.error) {
      errorMessage.value += `\n\nGraph API Error: ${JSON.stringify(
        error.response.data.data.error
      )}`;
    }
  } finally {
    isLoading.value = false;
  }
};

// Handle back button click
const goBack = () => {
  router.push(backRoute.value);
};

// Load status on mount
onMounted(() => {
  if (props.isEmbedded) {
    emit("configured");
  }
});
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div
      class="pt-4 px-6 flex justify-between items-center"
      v-if="showBackButton !== true"
    >
      <BackButton
        :fallbackRoute="backRoute"
        :label="
          configMode === 'graph'
            ? 'Back to Graph Connection'
            : 'Back to Authentication Providers'
        "
      />
    </div>

    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-[120rem]">
      <div class="flex flex-col gap-2" v-if="!isEmbedded">
        <h1 class="text-2xl font-bold text-white">
          Microsoft Entra Configuration
        </h1>
        <p class="text-slate-400">
          Follow this guide to set up Microsoft Entra ID authentication and
          Graph API access
        </p>
      </div>

      <!-- Responsive layout container -->
      <div class="flex flex-col lg:flex-row gap-6">
        <!-- Left column - Environment Variables and Testing -->
        <div class="flex flex-col gap-6 lg:w-1/2 lg:max-w-2xl">
          <!-- Environment Variables Section -->
          <div
            class="flex flex-col gap-4 bg-slate-800 border border-slate-700 rounded-lg p-6"
          >
            <div class="flex items-center gap-3">
              <div
                class="h-10 w-10 bg-blue-900/50 rounded-lg flex items-center justify-center"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-6 w-6 text-blue-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M3 4a1 1 0 011-1h3a1 1 0 011 1v3a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h3a1 1 0 011 1v3a1 1 0 01-1 1H4a1 1 0 01-1-1v-3zM10 4a1 1 0 011-1h3a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1V4zM10 10a1 1 0 011-1h3a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-3z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div>
                <h2 class="text-xl font-medium text-white">
                  Environment Configuration
                </h2>
                <p class="text-slate-400 text-sm">
                  Configure Microsoft Entra credentials via server environment
                  variables
                </p>
              </div>
            </div>

            <div
              class="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4"
            >
              <div class="flex flex-col gap-4">
                <div class="flex items-center gap-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-blue-300"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  <h3 class="text-blue-300 font-medium">
                    Required Environment Variables
                  </h3>
                </div>
                <p class="text-slate-400 text-sm">
                  Set these environment variables on your server for Microsoft
                  Entra authentication:
                </p>

                <div class="flex flex-col gap-3">
                  <div
                    class="bg-slate-800/60 rounded-md p-3 border border-slate-700"
                  >
                    <code class="text-green-400 font-mono text-sm"
                      >MICROSOFT_CLIENT_ID</code
                    >
                    <p class="text-slate-400 text-xs mt-1">
                      The Application (client) ID from your app registration
                    </p>
                  </div>
                  <div
                    class="bg-slate-800/60 rounded-md p-3 border border-slate-700"
                  >
                    <code class="text-green-400 font-mono text-sm"
                      >MICROSOFT_TENANT_ID</code
                    >
                    <p class="text-slate-400 text-xs mt-1">
                      Your Microsoft Entra directory (tenant) ID
                    </p>
                  </div>
                  <div
                    class="bg-slate-800/60 rounded-md p-3 border border-slate-700"
                  >
                    <code class="text-green-400 font-mono text-sm"
                      >MICROSOFT_CLIENT_SECRET</code
                    >
                    <p class="text-slate-400 text-xs mt-1">
                      The client secret value from your app registration
                    </p>
                  </div>
                  <div
                    class="bg-slate-800/60 rounded-md p-3 border border-slate-700"
                  >
                    <code class="text-green-400 font-mono text-sm"
                      >MICROSOFT_REDIRECT_URI</code
                    >
                    <p class="text-slate-400 text-xs mt-1">
                      The callback URL for OAuth authentication
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div
              class="bg-amber-900/20 border border-amber-700/50 rounded-lg p-4"
            >
              <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-amber-300"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  <h4 class="text-amber-300 font-medium">Security Note</h4>
                </div>
                <p class="text-amber-200 text-sm">
                  Store these values securely on your server. Never commit secrets
                  to version control. Consider using a secret management service
                  for production deployments.
                </p>
              </div>
            </div>
          </div>

          <!-- Testing Section -->
          <div
            class="flex flex-col gap-4 bg-slate-800 border border-slate-700 rounded-lg p-6"
          >
            <div class="flex flex-col gap-2">
              <div>
                <h3 class="text-lg font-medium text-white mb-4">
                  Test Configuration
                </h3>
                <p class="text-slate-400 mb-4">
                  Verify that your environment variables are properly configured
                </p>
              </div>

              <div class="flex gap-3 mb-4">
                <button
                  @click="testConfiguration"
                  class="px-4 py-2.5 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 font-medium"
                  :disabled="isLoading"
                >
                  <span v-if="isLoading" class="animate-spin h-4 w-4">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      ></circle>
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      ></path>
                    </svg>
                  </span>
                  {{ isLoading ? "Testing..." : "Test Authentication" }}
                </button>

                <button
                  @click="testAllPermissions"
                  class="px-4 py-2.5 bg-purple-700 text-white rounded-md hover:bg-purple-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 border border-purple-600"
                  :disabled="isLoading"
                >
                  <span v-if="isLoading" class="animate-spin h-4 w-4">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      ></circle>
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      ></path>
                    </svg>
                  </span>
                  {{ isLoading ? "Testing..." : "Test All Permissions" }}
                </button>
              </div>
            </div>

            <!-- Configuration Status -->
            <div
              v-if="configurationStatus"
              class="p-4 bg-green-900/30 border border-green-700/50 rounded-lg"
            >
              <div class="flex flex-col gap-2">
                <h4 class="text-green-300 font-medium">
                  Configuration Status
                </h4>
                <div class="flex flex-col gap-1 text-sm">
                  <div
                    v-if="configurationStatus.client_id_valid"
                    class="flex items-center gap-2 text-green-400"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-4 w-4"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                    Client ID is valid
                  </div>
                  <div
                    v-if="configurationStatus.tenant_id_valid"
                    class="flex items-center gap-2 text-green-400"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-4 w-4"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                    Tenant ID is valid
                  </div>
                  <div
                    v-if="configurationStatus.client_secret_valid"
                    class="flex items-center gap-2 text-green-400"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-4 w-4"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                    Client Secret is valid
                  </div>
                  <div
                    v-if="configurationStatus.redirect_uri_configured"
                    class="flex items-center gap-2 text-green-400"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-4 w-4"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                    Redirect URI is configured
                  </div>
                </div>
              </div>
            </div>

            <!-- Success Message -->
            <div
              v-if="successMessage"
              class="p-4 bg-green-900/30 text-green-400 rounded-lg border border-green-700/50"
            >
              <div class="flex items-start gap-3">
                <div class="mt-0.5 text-green-400">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
                <pre class="whitespace-pre-wrap text-sm font-normal">{{
                  successMessage
                }}</pre>
              </div>
            </div>

            <!-- Error Message -->
            <div
              v-if="errorMessage"
              class="p-4 bg-red-900/30 text-red-400 rounded-lg border border-red-700/50"
            >
              <div class="flex items-start gap-3">
                <div class="mt-0.5 text-red-400">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
                <pre class="whitespace-pre-wrap text-sm font-normal">{{
                  errorMessage
                }}</pre>
              </div>
            </div>

            <!-- Permission Test Results -->
            <div
              v-if="permissionTestResults"
              class="flex flex-col gap-4"
            >
              <div class="flex items-center gap-3">
                <h3 class="text-lg font-medium text-white">
                  Permission Test Results
                </h3>
                <div 
                  v-if="Object.values(permissionTestResults).every((r: any) => r.status === 'success')"
                  class="flex items-center gap-2 text-green-400 text-sm"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                  All permissions working
                </div>
                <div 
                  v-else
                  class="flex items-center gap-2 text-amber-400 text-sm"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                  Some permissions need attention
                </div>
              </div>

              <div class="grid gap-3">
                <!-- Loop through each permission test result -->
                <div
                  v-for="(result, permission) in permissionTestResults"
                  :key="permission"
                  class="p-4 rounded-lg border"
                  :class="{
                    'bg-green-900/30 border-green-700/50': result.status === 'success',
                    'bg-red-900/30 border-red-700/50': result.status === 'error'
                  }"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="flex-1">
                      <div class="flex items-center gap-3 mb-2">
                        <code 
                          class="text-sm font-mono px-2 py-1 rounded"
                          :class="{
                            'bg-green-900/50 text-green-300': result.status === 'success',
                            'bg-red-900/50 text-red-300': result.status === 'error'
                          }"
                        >
                          {{ permission }}
                        </code>
                        <span 
                          v-if="String(permission) === 'ProfilePhoto.Read.All'"
                          class="text-xs px-2 py-1 bg-blue-900/50 text-blue-300 rounded"
                        >
                          Optional
                        </span>
                        <span 
                          v-else
                          class="text-xs px-2 py-1 bg-red-900/50 text-red-300 rounded"
                        >
                          Required
                        </span>
                      </div>
                      
                      <div class="flex items-center gap-2 mb-2">
                        <svg
                          v-if="result.status === 'success'"
                          xmlns="http://www.w3.org/2000/svg"
                          class="h-5 w-5 text-green-400"
                          viewBox="0 0 20 20"
                          fill="currentColor"
                        >
                          <path
                            fill-rule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                            clip-rule="evenodd"
                          />
                        </svg>
                        <svg
                          v-else
                          xmlns="http://www.w3.org/2000/svg"
                          class="h-5 w-5 text-red-400"
                          viewBox="0 0 20 20"
                          fill="currentColor"
                        >
                          <path
                            fill-rule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                            clip-rule="evenodd"
                          />
                        </svg>
                        <span 
                          class="font-medium"
                          :class="{
                            'text-green-300': result.status === 'success',
                            'text-red-300': result.status === 'error'
                          }"
                        >
                          {{ result.status === 'success' ? 'Working' : 'Failed' }}
                        </span>
                      </div>

                      <!-- Success details -->
                      <div v-if="result.status === 'success' && result.data" class="text-sm text-slate-300">
                        <div v-if="result.data.count !== undefined">
                          Found {{ result.data.count }} records
                        </div>
                        <div v-if="result.data.photoAvailable === true && result.data.testedUser">
                          Profile photos accessible
                          <span class="text-slate-400">
                            (tested: {{ result.data.testedUser }})
                          </span>
                        </div>
                        <div v-if="result.data.photoAvailable === true && !result.data.testedUser">
                          Profile photo permission verified
                        </div>
                        <div v-if="result.data.photoAvailable === false">
                          Permission working (test user has no photo)
                        </div>
                        <div v-if="result.data.note" class="text-slate-400 italic text-xs mt-1">
                          {{ result.data.note }}
                        </div>
                      </div>

                      <!-- Error details -->
                      <div v-if="result.status === 'error'" class="text-sm text-red-300">
                        {{ result.error }}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Legacy Graph API Results (for backwards compatibility) -->
            <div
              v-if="graphApiResults && !permissionTestResults"
              class="p-4 bg-blue-900/30 text-blue-300 rounded-lg border border-blue-700/50"
            >
              <div class="flex flex-col gap-2">
                <h3 class="text-lg font-medium text-blue-200">
                  Graph API Results
                </h3>
                <pre
                  class="whitespace-pre-wrap text-sm font-mono bg-slate-800/50 p-3 rounded overflow-auto max-h-96 max-w-full break-words"
                  >{{ JSON.stringify(graphApiResults, null, 2) }}</pre
                >
              </div>
            </div>
          </div>
        </div>

        <!-- Right column - Setup Guide and Permissions -->
        <div class="flex flex-col gap-6 lg:w-1/2 lg:min-w-0">
          <!-- Setup Guide -->
          <div class="flex flex-col gap-6 bg-slate-800 border border-slate-700 rounded-lg p-6">
            <div class="flex items-center gap-3">
              <div
                class="h-10 w-10 bg-purple-900/50 rounded-lg flex items-center justify-center"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-6 w-6 text-purple-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M12.316 3.051a1 1 0 01.633 1.265l-4 12a1 1 0 11-1.898-.632l4-12a1 1 0 011.265-.633zM5.707 6.293a1 1 0 010 1.414L3.414 10l2.293 2.293a1 1 0 11-1.414 1.414l-3-3a1 1 0 010-1.414l3-3a1 1 0 011.414 0zm8.586 0a1 1 0 011.414 0l3 3a1 1 0 010 1.414l-3 3a1 1 0 11-1.414-1.414L16.586 10l-2.293-2.293a1 1 0 010-1.414z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div>
                <h2 class="text-xl font-medium text-white">
                  App Registration Setup
                </h2>
                <p class="text-slate-400 text-sm">
                  Step-by-step guide to create your Microsoft Entra app
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-6">
              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-blue-900/50 rounded-full flex items-center justify-center text-blue-400 font-medium text-sm"
                >
                  1
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">
                      Create App Registration
                    </h3>
                    <p class="text-sm text-slate-400">
                      Navigate to the <a href="https://entra.microsoft.com/">Microsoft Entra Portal</a> and create a new app
                      registration
                    </p>
                  </div>
                  <div
                    class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                  >
                    <p class="text-xs text-slate-300 mb-2">
                      Azure Portal Path:
                    </p>
                    <code class="text-blue-400 text-sm"
                      >Azure Portal → Microsoft Entra ID → App registrations →
                      New registration</code
                    >
                  </div>
                </div>
              </div>

              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-blue-900/50 rounded-full flex items-center justify-center text-blue-400 font-medium text-sm"
                >
                  2
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">
                      Configure Basic Settings
                    </h3>
                    <p class="text-sm text-slate-400">
                      Set up the basic application configuration
                    </p>
                  </div>
                  <div class="flex flex-col gap-2">
                    <div
                      class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                    >
                      <p class="text-xs text-slate-300 mb-1">Name:</p>
                      <code class="text-green-400 text-sm"
                        >Your App Name (e.g., "Nosdesk Authentication")</code
                      >
                    </div>
                    <div
                      class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                    >
                      <p class="text-xs text-slate-300 mb-1">
                        Supported account types:
                      </p>
                      <code class="text-green-400 text-sm"
                        >Accounts in this organizational directory only</code
                      >
                    </div>
                  </div>
                </div>
              </div>

              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-blue-900/50 rounded-full flex items-center justify-center text-blue-400 font-medium text-sm"
                >
                  3
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">Add Redirect URI</h3>
                    <p class="text-sm text-slate-400">
                      Configure the OAuth callback URL
                    </p>
                  </div>
                  <div
                    class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                  >
                    <p class="text-xs text-slate-300 mb-2">Platform: Web</p>
                    <code class="text-green-400 text-sm break-all"
                      >https://your-domain.com/auth/microsoft/callback</code
                    >
                  </div>
                </div>
              </div>

              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-blue-900/50 rounded-full flex items-center justify-center text-blue-400 font-medium text-sm"
                >
                  4
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">
                      Generate Client Secret
                    </h3>
                    <p class="text-sm text-slate-400">
                      Create a client secret for authentication
                    </p>
                  </div>

                  <div
                    class="bg-amber-900/20 border border-amber-700/50 rounded-md p-3"
                  >
                    <p class="text-amber-200 text-xs">
                      <strong>Important:</strong> Copy the secret value
                      immediately - it won't be shown again!
                    </p>
                  </div>
                  <div
                    class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                  >
                    <code class="text-blue-400 text-sm"
                      >Certificates & secrets → New client secret</code
                    >
                  </div>
                </div>
              </div>

              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-blue-900/50 rounded-full flex items-center justify-center text-blue-400 font-medium text-sm"
                >
                  5
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">
                      Configure API Permissions
                    </h3>
                    <p class="text-sm text-slate-400">
                      Add required Microsoft Graph permissions
                    </p>
                  </div>
                  <div class="flex flex-col gap-2">
                    <div
                      class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                    >
                      <code class="text-blue-400 text-sm"
                        >API permissions → Add a permission → Microsoft Graph</code
                      >
                    </div>
                    <div
                      class="bg-red-900/20 border border-red-700/50 rounded-md p-3"
                    >
                      <p class="text-red-200 text-xs">
                        <strong>Important:</strong> Add these as <strong>Application permissions</strong> (not Delegated):
                      </p>
                      <ul class="text-red-200 text-xs mt-2 pl-4 list-disc list-inside space-y-1">
                        <li>Directory.Read.All</li>
                        <li>Device.Read.All</li>
                        <li>User.Read.All</li>
                        <li>ProfilePhoto.Read.All (optional)</li>
                      </ul>
                    </div>
                  </div>
                </div>
              </div>

              <div class="flex gap-4">
                <div
                  class="flex-shrink-0 w-8 h-8 bg-red-600 rounded-full flex items-center justify-center text-white font-medium text-sm"
                >
                  6
                </div>
                <div class="flex flex-col gap-2">
                  <div>
                    <h3 class="text-white font-medium">
                      Grant Admin Consent (CRITICAL)
                    </h3>
                    <p class="text-sm text-slate-400">
                      This step is required for the API to work
                    </p>
                  </div>
                  <div class="flex flex-col gap-2">
                    <div
                      class="bg-slate-700/50 rounded-md p-3 border border-slate-600"
                    >
                      <code class="text-green-400 text-sm"
                        >Grant admin consent for [Your Organization]</code
                      >
                    </div>
                    <div
                      class="bg-red-900/20 border border-red-700/50 rounded-md p-3"
                    >
                      <p class="text-red-200 text-xs">
                        <strong>Required:</strong> You must click "Grant admin consent" for all the application permissions. Without this, you'll get "Authorization_RequestDenied" errors.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- API Permissions Section -->
          <div class="bg-slate-800 border border-slate-700 rounded-lg p-6">
            <div class="flex flex-col gap-4">
              <div class="flex items-center gap-3">
                <div
                  class="h-10 w-10 bg-green-900/50 rounded-lg flex items-center justify-center"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-6 w-6 text-green-400"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M4.083 9h1.946c.089-1.546.383-2.97.837-4.118A6.004 6.004 0 004.083 9zM10 2a8 8 0 100 16 8 8 0 000-16zm0 2c-.076 0-.232.032-.465.262-.238.234-.497.623-.737 1.182-.389.907-.673 2.142-.766 3.556h3.936c-.093-1.414-.377-2.649-.766-3.556-.24-.56-.5-.948-.737-1.182C10.232 4.032 10.076 4 10 4zm3.971 5c-.089-1.546-.383-2.97-.837-4.118A6.004 6.004 0 0115.917 9h-1.946zm-2.003 2H8.032c.093 1.414.377 2.649.766 3.556.24.56.5.948.737 1.182.233.23.389.262.465.262.076 0 .232-.032.465-.262.238-.234.498-.623.737-1.182.389-.907.673-2.142.766-3.556zm1.166 4.118c.454-1.147.748-2.572.837-4.118h1.946a6.004 6.004 0 01-2.783 4.118zm-6.268 0C6.412 13.97 6.118 12.546 6.03 11H4.083a6.004 6.004 0 002.783 4.118z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
                <div>
                  <h2 class="text-xl font-medium text-white">
                    Required API Permissions
                  </h2>
                  <p class="text-slate-400 text-sm">
                    Microsoft Graph permissions needed for authentication and data
                    access
                  </p>
                </div>
              </div>

              <div class="flex flex-col gap-4">
                <!-- Delegated Permissions -->
                <div
                  class="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4"
                >
                  <div class="flex flex-col gap-3">
                    <div class="flex items-center gap-3">
                      <span
                        class="bg-blue-900/50 text-blue-400 px-2 py-1 rounded text-xs"
                        >Delegated</span
                      >
                      <h3 class="text-blue-300 font-medium">
                        User Authentication Permissions
                      </h3>
                    </div>
                    <div class="flex flex-col gap-2">
                      <div
                        class="flex items-center justify-between p-2 bg-slate-800/50 rounded border border-slate-700"
                      >
                        <div>
                          <code class="text-green-400 text-sm">User.Read</code>
                          <p class="text-slate-400 text-xs mt-1">
                            Sign in and read user profile
                          </p>
                        </div>
                        <span class="text-green-400 text-xs">Required</span>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Application Permissions -->
                <div
                  class="bg-purple-900/20 border border-purple-700/50 rounded-lg p-4"
                >
                  <div class="flex flex-col gap-3">
                    <div class="flex items-center gap-3">
                      <span
                        class="bg-purple-900/50 text-purple-400 px-2 py-1 rounded text-xs"
                        >Application</span
                      >
                      <h3 class="text-purple-300 font-medium">
                        Graph API Data Access
                      </h3>
                    </div>
                    <div class="flex flex-col gap-2">
                      <div
                        class="flex items-center justify-between p-2 bg-slate-800/50 rounded border border-slate-700"
                      >
                        <div>
                          <code class="text-green-400 text-sm"
                            >Directory.Read.All</code
                          >
                          <p class="text-slate-400 text-xs mt-1">
                            Read directory data
                          </p>
                        </div>
                        <span class="text-green-400 text-xs">Required</span>
                      </div>
                      <div
                        class="flex items-center justify-between p-2 bg-slate-800/50 rounded border border-slate-700"
                      >
                        <div>
                          <code class="text-green-400 text-sm"
                            >Device.Read.All</code
                          >
                          <p class="text-slate-400 text-xs mt-1">
                            Read all devices
                          </p>
                        </div>
                        <span class="text-green-400 text-xs">Required</span>
                      </div>
                      <div
                        class="flex items-center justify-between p-2 bg-slate-800/50 rounded border border-slate-700"
                      >
                        <div>
                          <code class="text-green-400 text-sm">User.Read.All</code>
                          <p class="text-slate-400 text-xs mt-1">
                            Read all users' full profiles
                          </p>
                        </div>
                        <span class="text-green-400 text-xs">Required</span>
                      </div>
                      <div
                        class="flex items-center justify-between p-2 bg-slate-800/50 rounded border border-slate-700"
                      >
                        <div>
                          <code class="text-green-400 text-sm"
                            >ProfilePhoto.Read.All</code
                          >
                          <p class="text-slate-400 text-xs mt-1">
                            Read all profile photos
                          </p>
                        </div>
                        <span class="text-blue-400 text-xs">Optional</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Troubleshooting Section -->
          <div class="flex flex-col gap-4 bg-red-900/10 border border-red-700/30 rounded-lg p-6">
            <div class="flex items-center gap-3">
              <div
                class="h-10 w-10 bg-red-900/50 rounded-lg flex items-center justify-center"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-6 w-6 text-red-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div>
                <h2 class="text-xl font-medium text-white">
                  Common Issues & Solutions
                </h2>
                <p class="text-slate-400 text-sm">
                  Troubleshoot Microsoft Graph API errors
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-4">
              <div class="bg-red-900/20 border border-red-700/50 rounded-lg p-4">
                <div class="flex flex-col gap-3">
                  <h3 class="text-red-300 font-medium">
                    "Authorization_RequestDenied" Error
                  </h3>
                  <p class="text-slate-300 text-sm">
                    This error means your app registration lacks the required permissions or admin consent hasn't been granted.
                  </p>
                  <div class="flex flex-col gap-2">
                    <h4 class="text-red-200 font-medium text-sm">Solution:</h4>
                    <ol class="text-slate-300 text-sm pl-4 list-decimal list-inside flex flex-col gap-1">
                      <li>Go to your app registration → API permissions</li>
                      <li>Verify you have <strong>Application permissions</strong> (not Delegated) for:
                        <ul class="ml-4 pl-4 list-disc list-inside mt-1 space-y-1">
                          <li>Directory.Read.All</li>
                          <li>Device.Read.All</li>
                          <li>User.Read.All</li>
                        </ul>
                      </li>
                      <li>Click <strong>"Grant admin consent for [Your Organization]"</strong></li>
                      <li>Wait a few minutes for the permissions to propagate</li>
                    </ol>
                  </div>
                </div>
              </div>

              <div class="bg-amber-900/20 border border-amber-700/50 rounded-lg p-4">
                <div class="flex flex-col gap-3">
                  <h3 class="text-amber-300 font-medium">
                    "Insufficient privileges" Error
                  </h3>
                  <p class="text-slate-300 text-sm">
                    Your Azure account may not have permissions to grant admin consent.
                  </p>
                  <div class="flex flex-col gap-2">
                    <h4 class="text-amber-200 font-medium text-sm">Solution:</h4>
                    <ul class="text-slate-300 text-sm pl-4 list-disc list-inside flex flex-col gap-1">
                      <li>Contact your Azure administrator</li>
                      <li>Ask them to grant admin consent for the app registration</li>
                      <li>Or request Global Administrator or Application Administrator role</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
