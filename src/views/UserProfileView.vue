<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import StatusBadge from '@/components/StatusBadge.vue';
import UserAvatar from '@/components/UserAvatar.vue';
import BackButton from '@/components/common/BackButton.vue';
import { RouterLink } from 'vue-router';

interface UserProfile {
  username: string;
  email: string;
  role: string;
  department: string;
  joinedDate: string;
  avatar?: string | null;
}

interface Device {
  id: string;
  name: string;
  type: string;
  lastSeen: string;
  status: string;
}

interface Ticket {
  id: number;
  title: string;
  status: 'open' | 'in-progress' | 'closed';
  priority: string;
  created: string;
  modified: string;
  assignee: string;
  requester: string;
  device: {
    hostname: string;
    serialNumber: string;
    model: string;
    warrantyStatus: string;
  };
  notesAndComments: Array<{
    id: number;
    content: string;
    author: string;
    createdAt: string;
    attachments: Array<{
      url: string;
      name: string;
    }>;
  }>;
  articleContent: string;
}

const route = useRoute();
const router = useRouter();
const loading = ref(true);
const error = ref<string | null>(null);
const userProfile = ref<UserProfile | null>(null);
const assignedTickets = ref<Ticket[]>([]);
const requestedTickets = ref<Ticket[]>([]);
const devices = ref<Device[]>([]);

const fetchUserData = async () => {
  try {
    loading.value = true;
    error.value = null;
    
    // TODO: Replace with actual API calls
    // Mock data for now
    userProfile.value = {
      username: route.params.username as string,
      email: `${route.params.username}@company.com`,
      role: 'Support Engineer',
      department: 'IT Support',
      joinedDate: '2023-01-01',
    };

    // Load tickets from tickets.json
    const ticketData = (await import("@/data/tickets.json")).default;
    const allTickets = ticketData.tickets.map(ticket => ({
      ...ticket,
      status: ticket.status as 'open' | 'in-progress' | 'closed'
    })) as Ticket[];
    
    // Filter tickets for this user
    assignedTickets.value = allTickets.filter(t => 
      t.assignee?.toLowerCase() === route.params.username.toString().toLowerCase()
    );
    requestedTickets.value = allTickets.filter(t => 
      t.requester?.toLowerCase() === route.params.username.toString().toLowerCase()
    );

    // Mock devices data
    devices.value = [
      {
        id: '1',
        name: 'MacBook Pro',
        type: 'Laptop',
        lastSeen: new Date().toISOString(),
        status: 'online'
      },
      {
        id: '2',
        name: 'iPhone 13',
        type: 'Mobile',
        lastSeen: new Date(Date.now() - 3600000).toISOString(),
        status: 'away'
      }
    ];
  } catch (e) {
    error.value = 'Failed to load user profile';
    console.error('Error loading user profile:', e);
  } finally {
    loading.value = false;
  }
};

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffTime / (1000 * 60 * 60));
    const diffMinutes = Math.floor(diffTime / (1000 * 60));

    if (diffMinutes < 1) {
      return 'just now';
    } else if (diffMinutes < 60) {
      return `${diffMinutes} minute${diffMinutes === 1 ? '' : 's'} ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hour${diffHours === 1 ? '' : 's'} ago`;
    } else if (diffDays < 30) {
      return `${diffDays} day${diffDays === 1 ? '' : 's'} ago`;
    } else {
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      });
    }
  } catch (e) {
    return dateString;
  }
};

onMounted(() => {
  fetchUserData();
});
</script>

<template>
  <div class="flex-1">
    <div v-if="userProfile" class="flex flex-col">
      <!-- Navigation and actions bar -->
      <div class="pt-4 px-6 flex justify-between items-center">
        <BackButton fallbackRoute="/users" />
      </div>
      
      <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
        <!-- Grid Container -->
        <div class="grid-container">
          <!-- User Info Area -->
          <div class="info-area flex flex-col gap-4">
            <!-- User Profile Header -->
            <div class="bg-slate-800 rounded-2xl p-6">
              <div class="flex items-start gap-4">
                <!-- Avatar Container with fixed dimensions -->
                <div class="w-32 h-32 flex-shrink-0">
                  <UserAvatar
                    :name="userProfile.username"
                    size="full"
                    :avatar="userProfile.avatar"
                    :show-name="false"
                    :clickable="false"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <h1 class="text-2xl font-semibold text-white truncate">{{ userProfile.username }}</h1>
                  <p class="text-slate-400 truncate">{{ userProfile.email }}</p>
                  <div class="mt-2 flex items-center space-x-4">
                    <span class="text-sm text-slate-400">{{ userProfile.role }}</span>
                    <span class="text-slate-600">•</span>
                    <span class="text-sm text-slate-400">{{ userProfile.department }}</span>
                  </div>
                  <p class="mt-2 text-sm text-slate-500">
                    Joined {{ formatDate(userProfile.joinedDate) }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Devices Section -->
            <div class="bg-slate-800 rounded-2xl p-6">
              <h2 class="text-lg font-medium text-white mb-4">Devices</h2>
              <div v-if="devices.length === 0" class="text-slate-400 text-sm">
                No devices
              </div>
              <div v-else class="flex flex-col gap-2 space-y-4">
                <RouterLink
                  v-for="device in devices"
                  :key="device.id"
                  :to="`/devices/${device.id}`"
                  class="block bg-slate-700/50 p-3 rounded-lg hover:bg-slate-700 transition-colors"
                >
                  <div class="flex items-start justify-between">
                    <div>
                      <h3 class="font-medium text-white">{{ device.name }}</h3>
                      <p class="text-sm text-slate-400">{{ device.type }}</p>
                      <p class="text-xs text-slate-500">Last seen {{ formatDate(device.lastSeen) }}</p>
                    </div>
                    <StatusBadge type="status" :value="device.status === 'online' ? 'open' : device.status === 'away' ? 'in-progress' : 'closed'" />
                  </div>
                </RouterLink>
              </div>
            </div>
          </div>

          <!-- Tickets Area -->
          <div class="tickets-area flex flex-col gap-4">
            <!-- Assigned Tickets -->
            <div class="bg-slate-800 rounded-2xl p-6">
              <h2 class="text-lg font-medium text-white mb-4">Assigned Tickets</h2>
              <div v-if="assignedTickets.length === 0" class="text-slate-400 text-sm">
                No assigned tickets
              </div>
              <div v-else class="flex flex-col gap-2 space-y-4">
                <RouterLink
                  v-for="ticket in assignedTickets"
                  :key="ticket.id"
                  :to="`/tickets/${ticket.id}`"
                  class="block bg-slate-700/50 p-3 rounded-lg hover:bg-slate-700 transition-colors"
                >
                  <div class="flex items-start justify-between">
                    <div>
                      <h3 class="font-medium text-white">{{ ticket.title }}</h3>
                      <p class="text-sm text-slate-400">{{ formatDate(ticket.created) }}</p>
                    </div>
                    <StatusBadge type="status" :value="ticket.status as 'open' | 'in-progress' | 'closed'" />
                  </div>
                </RouterLink>
              </div>
            </div>

            <!-- Requested Tickets -->
            <div class="bg-slate-800 rounded-2xl p-6">
              <h2 class="text-lg font-medium text-white mb-4">Requested Tickets</h2>
              <div v-if="requestedTickets.length === 0" class="text-slate-400 text-sm">
                No requested tickets
              </div>
              <div v-else class="space-y-4">
                <RouterLink
                  v-for="ticket in requestedTickets"
                  :key="ticket.id"
                  :to="`/tickets/${ticket.id}`"
                  class="block bg-slate-700/50 p-3 rounded-lg hover:bg-slate-700 transition-colors"
                >
                  <div class="flex items-start justify-between">
                    <div>
                      <h3 class="font-medium text-white">{{ ticket.title }}</h3>
                      <p class="text-sm text-slate-400">{{ formatDate(ticket.created) }}</p>
                    </div>
                    <StatusBadge type="status" :value="ticket.status as 'open' | 'in-progress' | 'closed'" />
                  </div>
                </RouterLink>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="loading" class="flex justify-center items-center min-h-[200px]">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
    </div>

    <div v-else class="p-6 text-center text-slate-400">
      User not found
    </div>
  </div>
</template>

<style scoped>
.grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-template-areas: "info" "tickets";
  gap: 1rem;

  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas: "info tickets";
  }
}

.info-area {
  grid-area: info;
}

.tickets-area {
  grid-area: tickets;
}
</style> 