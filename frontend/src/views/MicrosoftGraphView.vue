<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import axios from "axios";
import BackButton from "@/components/common/BackButton.vue";
import Modal from "@/components/Modal.vue";
import MicrosoftConfigView from "@/views/MicrosoftConfigView.vue";

// Connection state
const connectionStatus = ref<
  "connected" | "disconnected" | "connecting" | "error"
>("disconnected");
const lastSync = ref<string | null>(null);
const errorMessage = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const isLoading = ref(false);
const configurationStatus = ref<any>(null);
const graphApiResults = ref<any>(null);
const permissionTestResults = ref<any>(null);
const syncResults = ref<any>(null);

// Connection details
const graphConfig = ref({
  clientId: "",
  clientSecret: "",
  tenantId: "",
  scopes: ["User.Read", "Directory.Read.All", "Device.Read.All"],
});

// Auth provider state
const microsoftAuthProviderFound = ref(false);
const microsoftAuthProviderId = ref<string | null>(null);

// Import options
const selectedEntities = ref(["devices", "users"]);
const availableEntities = [
  {
    id: "devices",
    name: "Devices",
    description: "Import device information from Intune",
  },
  {
    id: "users",
    name: "Users",
    description: "Import user information from Azure AD",
  },
  {
    id: "groups",
    name: "Groups",
    description: "Import security and distribution groups",
  },
];

// Modals
const showConfigModal = ref(false);
const showSyncModal = ref(false);

const router = useRouter();

// Fetch Microsoft Entra auth providers
const fetchMicrosoftAuthProvider = async () => {
  try {
    const response = await axios.get("/api/auth/providers");

    if (response.data && Array.isArray(response.data)) {
      // Find Microsoft Entra provider
      const microsoftProvider = response.data.find(
        (p: { provider_type: string; enabled: boolean }) =>
          p.provider_type === "microsoft" && p.enabled
      );

      if (microsoftProvider) {
        microsoftAuthProviderFound.value = true;
        microsoftAuthProviderId.value = microsoftProvider.id;
        return microsoftProvider;
      }
    }

    return null;
  } catch (error) {
    console.error("Failed to fetch Microsoft Auth Provider:", error);
    return null;
  }
};

// Fetch current connection status
const fetchConnectionStatus = async () => {
  isLoading.value = true;

  try {
    // Use the real Microsoft Graph integration endpoint
    const response = await axios.get("/api/integrations/graph/status");
    connectionStatus.value = response.data.status;

    if (response.data.last_sync) {
      lastSync.value = new Date(response.data.last_sync).toLocaleString();
    }

    // Load connection details
    if (connectionStatus.value === "connected") {
      graphConfig.value = {
        ...graphConfig.value,
        clientId: response.data.client_id,
        tenantId: response.data.tenant_id,
        // We don't display the client secret for security reasons
        clientSecret: "",
      };
    }

    // Check if we have a Microsoft auth provider configured
    await fetchMicrosoftAuthProvider();
  } catch (error) {
    console.error("Failed to fetch MS Graph connection status:", error);
    errorMessage.value = "Failed to fetch connection status";
    connectionStatus.value = "error";

    // Even if we can't get the connection status, still check for Microsoft auth provider
    await fetchMicrosoftAuthProvider();
  } finally {
    isLoading.value = false;
  }
};

// Configure connection
const configureConnection = () => {
  if (microsoftAuthProviderId.value) {
    router.push(`/admin/microsoft-config/${microsoftAuthProviderId.value}?mode=graph`);
  } else {
    errorMessage.value = "No Microsoft Auth Provider found. Please set one up first.";
    setTimeout(() => {
      errorMessage.value = null;
    }, 3000);
  }
};

// Test connection
const testConnection = async () => {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    // Use the real Microsoft Graph integration endpoint
    const response = await axios.post("/api/integrations/graph/test");

    if (response.data.success) {
      connectionStatus.value = "connected";
      successMessage.value = response.data.message || "Connection tested successfully";
    } else {
      connectionStatus.value = "error";
      errorMessage.value = response.data.message || "Connection test failed";
    }

    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (error: any) {
    console.error("Failed to test connection:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to test connection";
    connectionStatus.value = "error";
  } finally {
    isLoading.value = false;
  }
};

// Sync data
const syncData = () => {
  showSyncModal.value = true;
};

// Sync profile photos
const syncProfilePhotos = async () => {
  isLoading.value = true;
  errorMessage.value = null;
  syncResults.value = null;

  try {
    const response = await axios.post("/api/integrations/graph/sync-photos");

    if (response.data.success) {
      successMessage.value = response.data.message || "Profile photos synced successfully";
      
      // Store detailed sync results
      syncResults.value = {
        success: true,
        message: response.data.message,
        results: [{
          entity: "profile_photos",
          processed: response.data.photos_synced,
          total: response.data.total_users,
          status: response.data.photos_failed > 0 ? "completed_with_errors" : "completed",
          errors: response.data.errors || []
        }],
        total_processed: response.data.photos_synced,
        total_errors: response.data.photos_failed
      };
      
      console.log("Profile photo sync results:", response.data);
    } else {
      errorMessage.value = response.data.message || "Failed to sync profile photos";
    }

    setTimeout(() => {
      successMessage.value = null;
    }, 5000);
  } catch (error: any) {
    console.error("Failed to sync profile photos:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to sync profile photos";
  } finally {
    isLoading.value = false;
  }
};

// Start sync process
const startSync = async () => {
  isLoading.value = true;
  errorMessage.value = null;
  syncResults.value = null; // Clear previous results

  try {
    // Use the real Microsoft Graph integration endpoint
    const response = await axios.post("/api/integrations/graph/sync", {
      entities: selectedEntities.value,
    });

    showSyncModal.value = false;

    if (response.data.success) {
      successMessage.value = response.data.message || "Sync completed successfully";
      lastSync.value = new Date().toLocaleString();
      
      // Store detailed sync results
      syncResults.value = response.data;
      
      // Log sync details for debugging
      console.log("Sync results:", response.data);
    } else {
      errorMessage.value = response.data.message || "Failed to start sync";
      // Store failed sync results too for debugging
      if (response.data.results) {
        syncResults.value = response.data;
      }
    }

    setTimeout(() => {
      successMessage.value = null;
    }, 5000); // Show success message longer for sync operations
  } catch (error: any) {
    console.error("Failed to start sync:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to start sync";
  } finally {
    isLoading.value = false;
  }
};

// Toggle entity selection
const toggleEntity = (entityId: string) => {
  if (selectedEntities.value.includes(entityId)) {
    selectedEntities.value = selectedEntities.value.filter(
      (id) => id !== entityId
    );
  } else {
    selectedEntities.value.push(entityId);
  }
};

// Format status for display
const getStatusDisplay = (status: string) => {
  switch (status) {
    case "connected":
      return {
        text: "Connected",
        class: "bg-green-900/50 text-green-400 border-green-700",
      };
    case "disconnected":
      return {
        text: "Not Connected",
        class: "bg-slate-900/50 text-slate-400 border-slate-700",
      };
    case "connecting":
      return {
        text: "Connecting...",
        class: "bg-blue-900/50 text-blue-400 border-blue-700",
      };
    case "error":
      return {
        text: "Connection Error",
        class: "bg-red-900/50 text-red-400 border-red-700",
      };
    default:
      return {
        text: "Unknown",
        class: "bg-slate-900/50 text-slate-400 border-slate-700",
      };
  }
};

// Handle configuration completion
const handleConfigured = async () => {
  showConfigModal.value = false;
  await fetchConnectionStatus();
};

onMounted(() => {
  fetchConnectionStatus();
});
</script>

<template>
  <div class="flex-1">
    <!-- Navigation and actions bar -->
    <div class="pt-4 px-6 flex justify-between items-center">
      <BackButton
        fallbackRoute="/admin/data-import"
        label="Back to Data Import"
      />
    </div>

    <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
      <div class="mb-6">
        <h1 class="text-2xl font-bold text-white">
          Microsoft Graph Connection
        </h1>
        <p class="text-slate-400 mt-2">
          Configure and manage the connection to Microsoft Graph API for
          importing data from Microsoft services
        </p>
      </div>

      <!-- Success message -->
      <div
        v-if="successMessage"
        class="p-4 bg-green-900/50 text-green-400 rounded-lg border border-green-700"
      >
        {{ successMessage }}
      </div>

      <!-- Error message -->
      <div
        v-if="errorMessage"
        class="p-4 bg-red-900/50 text-red-400 rounded-lg border border-red-700"
      >
        {{ errorMessage }}
      </div>

      <!-- Connection status card -->
      <div class="bg-slate-800 border border-slate-700 rounded-lg p-6 mb-4">
        <div
          class="flex flex-col md:flex-row md:justify-between md:items-center gap-4"
        >
          <div class="flex flex-col gap-1">
            <h2 class="text-xl font-medium text-white mb-2">
              Connection Status
            </h2>
            <div class="flex items-center">
              <span
                :class="[
                  'px-3 py-1 rounded-full text-sm inline-flex items-center border gap-1',
                  getStatusDisplay(connectionStatus).class,
                ]"
              >
                <span
                  class="h-2 w-2 rounded-full mr-2"
                  :class="{
                    'bg-green-400': connectionStatus === 'connected',
                    'bg-slate-400': connectionStatus === 'disconnected',
                    'bg-blue-400': connectionStatus === 'connecting',
                    'bg-red-400': connectionStatus === 'error',
                  }"
                ></span>
                {{ getStatusDisplay(connectionStatus).text }}
              </span>
            </div>
            <p v-if="lastSync" class="text-sm text-slate-400 mt-2">
              Last synchronized: {{ lastSync }}
            </p>
          </div>

          <div class="flex flex-wrap gap-3">
            <button
              @click="configureConnection"
              class="px-4 py-2 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors border border-slate-600 flex items-center gap-2"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
              Configure
            </button>

            <button
              @click="testConnection"
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
              :disabled="isLoading"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13 10V3L4 14h7v7l9-11h-7z"
                />
              </svg>
              Test Connection
            </button>

            <button
              @click="syncData"
              class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2"
              :disabled="connectionStatus !== 'connected' || isLoading"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              Sync Data
            </button>

            <button
              @click="syncProfilePhotos"
              class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors flex items-center gap-2"
              :disabled="connectionStatus !== 'connected' || isLoading"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                />
              </svg>
              Sync Profile Photos (64x64)
            </button>
          </div>
        </div>
      </div>

      <!-- Available entities -->
      <div
        class="flex flex-col gap-2 bg-slate-800 border border-slate-700 rounded-lg p-6"
      >
        <div>
          <h2 class="text-xl font-medium text-white mb-4">
            Available Data Entities
          </h2>
          <p class="text-slate-400 mb-4">
            These are the data entities that can be imported from Microsoft
            Graph API
          </p>
        </div>

        <div class="flex flex-col gap-4">
          <div
            v-for="entity in availableEntities"
            :key="entity.id"
            class="p-4 bg-slate-700 rounded-lg border border-slate-600"
          >
            <div class="flex items-start">
              <div class="flex-1">
                <h3 class="text-white font-medium">{{ entity.name }}</h3>
                <p class="text-sm text-slate-400 mt-1">
                  {{ entity.description }}
                </p>
              </div>
              <div class="ml-4">
                <button
                  @click="toggleEntity(entity.id)"
                  class="px-3 py-1 text-sm rounded-md"
                  :class="
                    selectedEntities.includes(entity.id)
                      ? 'bg-blue-900/50 text-blue-400 border border-blue-700'
                      : 'bg-slate-800 text-slate-400 border border-slate-700'
                  "
                >
                  {{
                    selectedEntities.includes(entity.id) ? "Selected" : "Select"
                  }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Configure Connection Modal -->
    <!-- Remove or comment out this modal since we're using a separate page now -->
    <!-- <Modal
      :show="showConfigModal"
      title="Configure Microsoft Graph Connection"
      contentClass="max-w-6xl"
      @close="showConfigModal = false"
    >
      <MicrosoftConfigView
        mode="graph"
        :showBackButton="false"
        @configured="handleConfigured"
      />
    </Modal> -->

    <!-- Sync Data Modal -->
    <Modal
      :show="showSyncModal"
      title="Sync Data from Microsoft Graph"
      contentClass="max-w-lg"
      @close="showSyncModal = false"
    >
      <div class="space-y-4">
        <p class="text-slate-300">
          Select the data entities you want to import from Microsoft Graph:
        </p>

        <div class="space-y-2">
          <div
            v-for="entity in availableEntities"
            :key="entity.id"
            class="flex items-center p-3 rounded-md border border-slate-600 bg-slate-700/50"
          >
            <input
              type="checkbox"
              :id="`sync-${entity.id}`"
              :checked="selectedEntities.includes(entity.id)"
              @change="toggleEntity(entity.id)"
              class="w-4 h-4 text-blue-600 bg-slate-700 border-slate-600 focus:ring-blue-500 focus:ring-2"
            />
            <label
              :for="`sync-${entity.id}`"
              class="ml-3 block text-white cursor-pointer"
            >
              {{ entity.name }}
            </label>
          </div>
        </div>

        <div
          class="bg-blue-900/20 border border-blue-800/50 rounded-md p-3 text-sm text-blue-300"
        >
          <p class="flex items-start">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 mr-2 flex-shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span>
              Synchronization will import the latest data from Microsoft
              services. This may take several minutes depending on the amount of
              data.
            </span>
          </p>
        </div>

        <!-- Sync Progress Results -->
        <div 
          v-if="syncResults" 
          class="mt-4 p-4 bg-slate-700 rounded-lg border border-slate-600"
        >
          <h4 class="text-lg font-medium text-white mb-3">Sync Results</h4>
          
          <div class="space-y-3">
            <div v-for="result in syncResults.results" :key="result.entity" class="p-3 rounded-md border" :class="{
              'bg-green-900/30 border-green-700': result.status === 'completed',
              'bg-yellow-900/30 border-yellow-700': result.status === 'completed_with_errors',
              'bg-red-900/30 border-red-700': result.status === 'error'
            }">
              <div class="flex justify-between items-start">
                <div>
                  <h5 class="font-medium text-white capitalize">{{ result.entity }}</h5>
                  <p class="text-sm text-slate-300">
                    Processed {{ result.processed }} of {{ result.total }} items
                    <span v-if="result.total > 0" class="ml-2 text-slate-400">
                      ({{ Math.round((result.processed / result.total) * 100) }}%)
                    </span>
                  </p>
                </div>
                <div class="flex items-center">
                  <svg v-if="result.status === 'completed'" class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                  </svg>
                  <svg v-else-if="result.status === 'completed_with_errors'" class="w-5 h-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.864-.833-2.634 0l-5.898 8.5c-.77.833.192 2.5 1.732 2.5z"></path>
                  </svg>
                  <svg v-else class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                  </svg>
                </div>
              </div>
              
              <!-- Error Details -->
              <div v-if="result.errors && result.errors.length > 0" class="mt-2">
                <p class="text-sm font-medium text-red-300 mb-1">Errors:</p>
                <ul class="text-sm text-red-200 space-y-1">
                  <li v-for="error in result.errors.slice(0, 3)" :key="error" class="pl-2 border-l-2 border-red-500">
                    {{ error }}
                  </li>
                  <li v-if="result.errors.length > 3" class="text-red-300 italic">
                    ... and {{ result.errors.length - 3 }} more errors
                  </li>
                </ul>
              </div>
            </div>
          </div>
          
          <!-- Overall Summary -->
          <div class="mt-4 pt-3 border-t border-slate-600">
            <div class="flex justify-between items-center text-sm">
              <span class="text-slate-300">Total processed:</span>
              <span class="font-medium text-white">{{ syncResults.total_processed }} items</span>
            </div>
            <div v-if="syncResults.total_errors > 0" class="flex justify-between items-center text-sm mt-1">
              <span class="text-slate-300">Total errors:</span>
              <span class="font-medium text-red-400">{{ syncResults.total_errors }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="flex justify-end gap-3 mt-6">
        <button
          @click="showSyncModal = false"
          class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 border border-slate-600"
        >
          Cancel
        </button>
        <button
          @click="startSync"
          class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
          :disabled="isLoading || selectedEntities.length === 0"
        >
          <span v-if="isLoading">Starting Sync...</span>
          <span v-else>Start Sync</span>
        </button>
      </div>
    </Modal>
  </div>
</template>
