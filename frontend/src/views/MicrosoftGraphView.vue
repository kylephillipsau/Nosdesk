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
const showImportCredentialsModal = ref(false);

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

// Import credentials from Microsoft Entra auth provider
const importCredentialsFromAuthProvider = async () => {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    // First check if we have a Microsoft Auth Provider
    const microsoftProvider = await fetchMicrosoftAuthProvider();

    if (!microsoftProvider) {
      errorMessage.value = "No configured Microsoft Entra provider found";
      return;
    }

    // Get the configs for this provider
    if (microsoftProvider.configs && Array.isArray(microsoftProvider.configs)) {
      // Extract client ID and tenant ID
      const clientIdConfig = microsoftProvider.configs.find(
        (c: { key: string; value: string }) => c.key === "client_id"
      );
      const tenantIdConfig = microsoftProvider.configs.find(
        (c: { key: string; value: string }) => c.key === "tenant_id"
      );

      if (clientIdConfig && tenantIdConfig) {
        // Update our graph config
        graphConfig.value.clientId = clientIdConfig.value;
        graphConfig.value.tenantId = tenantIdConfig.value;

        // Client secret won't be returned for security reasons
        graphConfig.value.clientSecret = "";

        successMessage.value =
          "Successfully imported Microsoft Entra credentials";
        setTimeout(() => {
          successMessage.value = null;
        }, 3000);

        // Close the modal if it's open
        showImportCredentialsModal.value = false;

        // Save the imported configuration
        await saveConfiguration();
      } else {
        errorMessage.value =
          "Microsoft Entra provider is missing required configuration";
      }
    } else {
      errorMessage.value = "Microsoft Entra provider has no configuration";
    }
  } catch (error: any) {
    console.error("Failed to import credentials:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to import credentials";
  } finally {
    isLoading.value = false;
  }
};

// Fetch current connection status
const fetchConnectionStatus = async () => {
  isLoading.value = true;

  try {
    // This is a placeholder - replace with actual API endpoint
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
  showConfigModal.value = true;
};

// Show import credentials modal
const showImportCredentials = () => {
  showImportCredentialsModal.value = true;
};

// Save connection configuration
const saveConfiguration = async () => {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    // This is a placeholder - replace with actual API endpoint
    await axios.post("/api/integrations/graph/configure", {
      client_id: graphConfig.value.clientId,
      tenant_id: graphConfig.value.tenantId,
      client_secret: graphConfig.value.clientSecret,
      scopes: graphConfig.value.scopes.join(" "),
    });

    showConfigModal.value = false;

    // Show success message
    successMessage.value = "Configuration saved successfully";
    setTimeout(() => {
      successMessage.value = null;
    }, 3000);

    // Refresh connection status
    await fetchConnectionStatus();
  } catch (error: any) {
    console.error("Failed to save configuration:", error);
    errorMessage.value =
      error.response?.data?.message || "Failed to save configuration";
  } finally {
    isLoading.value = false;
  }
};

// Test connection
const testConnection = async () => {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    // This is a placeholder - replace with actual API endpoint
    const response = await axios.post("/api/integrations/graph/test");

    if (response.data.success) {
      connectionStatus.value = "connected";
      successMessage.value = "Connection tested successfully";
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

// Start sync process
const startSync = async () => {
  isLoading.value = true;
  errorMessage.value = null;

  try {
    // This is a placeholder - replace with actual API endpoint
    const response = await axios.post("/api/integrations/graph/sync", {
      entities: selectedEntities.value,
    });

    showSyncModal.value = false;

    if (response.data.success) {
      successMessage.value = "Sync started successfully";
      lastSync.value = new Date().toLocaleString();
    } else {
      errorMessage.value = response.data.message || "Failed to start sync";
    }

    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
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

      <!-- Microsoft Auth Provider info message -->
      <div
        v-if="microsoftAuthProviderFound"
        class="p-4 bg-blue-900/50 text-blue-400 rounded-lg border border-blue-700 flex items-start gap-2"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5 flex-shrink-0 mt-0.5"
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
        <div>
          <p>
            Microsoft Entra authentication provider is configured. You can
            <button
              @click="showImportCredentials"
              class="text-blue-300 underline hover:text-blue-200"
            >
              import these credentials
            </button>
            to use with Microsoft Graph.
          </p>
        </div>
      </div>

      <!-- Connection status card -->
      <div class="bg-slate-800 border border-slate-700 rounded-lg p-6 mb-4">
        <div
          class="flex flex-col md:flex-row md:justify-between md:items-center gap-4"
        >
          <div>
            <h2 class="text-xl font-medium text-white mb-2">
              Connection Status
            </h2>
            <div class="flex items-center">
              <span
                :class="[
                  'px-3 py-1 rounded-full text-sm inline-flex items-center border',
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
                  d="M12 8a4 4 0 100 8 4 4 0 000-8zm0 0v-1.5a2 2 0 012-2h1.5l.5 1.5c.3.9.8 1.7 1.5 2.3l1.2-.5 1 1.7-1.2.9c.2.7.2 1.4 0 2.1l1.2.9-1 1.7-1.2-.5c-.7.6-1.2 1.4-1.5 2.3l-.5 1.5h-1.5a2 2 0 01-2-2v-1.5a6 6 0 00-2.3-1.5l-.9 1.2-1.7-1 1-.9a4 4 0 010-2.1l-1-.9 1-1.7.9 1.2a6 6 0 002.3-1.5l.5-1.5h1.5a2 2 0 012 2z"
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
          </div>
        </div>
      </div>

      <!-- Available entities -->
      <div class="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <h2 class="text-xl font-medium text-white mb-4">
          Available Data Entities
        </h2>
        <p class="text-slate-400 mb-4">
          These are the data entities that can be imported from Microsoft Graph
          API
        </p>

        <div class="space-y-4">
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
    <Modal
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
    </Modal>

    <!-- Import Credentials Modal -->
    <Modal
      :show="showImportCredentialsModal"
      title="Import Microsoft Entra Credentials"
      contentClass="max-w-lg"
      @close="showImportCredentialsModal = false"
    >
      <div class="space-y-4">
        <p class="text-slate-300">
          Import credentials from your configured Microsoft Entra authentication
          provider. This will use the same app registration for Microsoft Graph
          API.
        </p>

        <div
          class="bg-blue-900/20 border border-blue-800/50 rounded-md p-4 text-blue-300"
        >
          <p class="flex items-start gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 flex-shrink-0"
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
              To use Microsoft Graph API, your app registration in Microsoft
              Entra ID must also have the required Graph API permissions added
              and granted.
            </span>
          </p>
        </div>

        <div
          class="bg-amber-900/20 border border-amber-800/50 rounded-md p-4 text-amber-300"
        >
          <p class="flex items-start gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5 flex-shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <span>
              You will need to provide the client secret again, as it is not
              stored in a retrievable format.
            </span>
          </p>
        </div>
      </div>

      <div class="flex justify-end gap-3 mt-6">
        <button
          @click="showImportCredentialsModal = false"
          class="px-4 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-600 border border-slate-600"
        >
          Cancel
        </button>
        <button
          @click="importCredentialsFromAuthProvider"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
          :disabled="isLoading || !microsoftAuthProviderFound"
        >
          <span v-if="isLoading">Importing...</span>
          <span v-else>Import Credentials</span>
        </button>
      </div>
    </Modal>

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
