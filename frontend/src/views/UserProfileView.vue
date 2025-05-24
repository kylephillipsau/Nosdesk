<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import StatusBadge from '@/components/StatusBadge.vue';
import UserAvatar from '@/components/UserAvatar.vue';
import BackButton from '@/components/common/BackButton.vue';
import { RouterLink } from 'vue-router';
import ticketService from '@/services/ticketService';
import userService from '@/services/userService';
import type { Ticket } from '@/services/ticketService';
import type { User } from '@/services/userService';

interface UserProfile extends User {
  department?: string;
  joinedDate?: string;
  created_at?: string;
}

interface Device {
  id: string;
  name: string;
  type: string;
  lastSeen: string;
}

const route = useRoute();
const router = useRouter();
const loading = ref(true);
const error = ref<string | null>(null);
const userProfile = ref<UserProfile | null>(null);
const assignedTickets = ref<Ticket[]>([]);
const requestedTickets = ref<Ticket[]>([]);
const devices = ref<Device[]>([]);

// Update document title when user profile changes
watch(userProfile, (newProfile) => {
  if (newProfile) {
    document.title = `${newProfile.name}'s Profile | Nosdesk`;
  }
});

const fetchUserData = async () => {
  try {
    loading.value = true;
    error.value = null;
    
    // Get the UUID from the route params
    const userUuid = route.params.uuid as string;
    
    if (!userUuid) {
      error.value = 'User ID is missing';
      return;
    }
    
    // Fetch the user from the API
    const user = await userService.getUserByUuid(userUuid);
    
    if (!user) {
      error.value = 'User not found';
      return;
    }
    
    // Create the user profile with the fetched data
    userProfile.value = {
      ...user,
      department: 'IT Support', // Default department (could be added to backend later)
      joinedDate: new Date().toISOString() // Default to current date
    };

    // Load tickets from the API
    const allTickets = await ticketService.getTickets();
    
    // Filter tickets for this user
    assignedTickets.value = allTickets.filter(t => 
      t.assignee === userUuid
    );
    requestedTickets.value = allTickets.filter(t => 
      t.requester === userUuid
    );

    // Mock devices data - this could be replaced with real API data later
    devices.value = [
      {
        id: '1',
        name: 'MacBook Pro',
        type: 'Laptop',
        lastSeen: new Date().toISOString()
      },
      {
        id: '2',
        name: 'iPhone 13',
        type: 'Mobile',
        lastSeen: new Date(Date.now() - 3600000).toISOString()
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
        <BackButton fallbackRoute="/users" label="Back to Users" />
        <RouterLink 
          :to="`/profile/settings`"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
        >
          Edit Profile
        </RouterLink>
      </div>
      
      <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
        <!-- Grid Container -->
        <div class="grid-container">
          <!-- User Info Area -->
          <div class="info-area flex flex-col gap-4">
            <!-- User Profile Card with Banner -->
            <div class="bg-slate-800 rounded-2xl overflow-hidden">
              <!-- Cover/Banner Image -->
              <div 
                class="h-42 bg-gradient-to-r from-blue-600 to-purple-600 relative"
                :style="userProfile.banner_url ? `background-image: url('${userProfile.banner_url}'); background-size: cover; background-position: center;` : ''"
              >
              </div>
              
              <!-- Profile Content -->
              <div class="px-6 pt-16 pb-6 relative">
                <!-- Avatar that overlaps the banner -->
                <div class="absolute -top-16 left-6 w-32 h-32 rounded-full overflow-hidden border-4 border-slate-800">
                  <UserAvatar
                    :name="userProfile.name"
                    size="full"
                    :avatar="userProfile.avatar_url || null"
                    :showName="false"
                    :clickable="false"
                    class="w-full h-full"
                  />
                </div>
                
                <!-- User Info -->
                <div class="flex justify-between items-start">
                  <div>
                    <h1 class="text-2xl font-semibold text-white truncate">{{ userProfile.name }}</h1>
                    <p class="text-slate-400 truncate">{{ userProfile.email }}</p>
                    <div class="mt-2 flex items-center space-x-4">
                      <span class="text-sm text-slate-400">{{ userProfile.role }}</span>
                      <span class="text-slate-600">•</span>
                      <span class="text-sm text-slate-400">{{ userProfile.department }}</span>
                    </div>
                    <div v-if="userProfile.pronouns" class="mt-1">
                      <span class="text-sm text-slate-500">{{ userProfile.pronouns }}</span>
                    </div>
                    <p class="mt-2 text-sm text-slate-500">
                      Joined {{ userProfile.joinedDate ? formatDate(userProfile.joinedDate) : 'Recently' }}
                    </p>
                  </div>
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
                  </div>
                </RouterLink>
              </div>
            </div>
          </div>

          <!-- Tickets Area -->
          <div class="tickets-area flex flex-col gap-4">
            <!-- Assigned Tickets -->
            <div class="flex flex-col gap-2 bg-slate-800 rounded-2xl p-6">
              <div class="flex justify-between items-center mb-4">
                <h2 class="text-lg font-medium text-white">{{ assignedTickets.length }} Assigned Tickets</h2>
                <RouterLink 
                  :to="`/tickets?assignee=${route.params.uuid}`"
                  class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
                >
                  See All
                </RouterLink>
              </div>
              <div v-if="assignedTickets.length === 0" class="text-slate-400 text-sm">
                No assigned tickets
              </div>
              <div v-else class="flex flex-col gap-2 space-y-4">
                <RouterLink
                  v-for="ticket in assignedTickets.slice(0, 5)"
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
            <div class="flex flex-col gap-2 bg-slate-800 rounded-2xl p-6">
              <div class="flex justify-between items-center mb-4">
                <h2 class="text-lg font-medium text-white">{{ requestedTickets.length }} Requested Tickets</h2>
                <RouterLink 
                  :to="`/tickets?requester=${route.params.uuid}`"
                  class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
                >
                  See All
                </RouterLink>
              </div>
              <div v-if="requestedTickets.length === 0" class="text-slate-400 text-sm">
                No requested tickets
              </div>
              <div v-else class="flex flex-col gap-2 space-y-4">
                <RouterLink
                  v-for="ticket in requestedTickets.slice(0, 5)"
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