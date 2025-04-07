<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import ticketService, { type Ticket } from '@/services/ticketService';

const props = defineProps({
  limit: {
    type: Number,
    default: 5
  },
  showTitle: {
    type: Boolean,
    default: true
  },
  filterStatus: {
    type: String,
    default: '' // empty string means show all
  }
});

const router = useRouter();
const authStore = useAuthStore();

const tickets = ref<Ticket[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedStatus = ref(props.filterStatus);
const sortBy = ref('date'); // default sort by date

// Status options for the filter
const statusOptions = [
  { value: '', label: 'All' },
  { value: 'open', label: 'Open' },
  { value: 'in-progress', label: 'In Progress' },
  { value: 'closed', label: 'Closed' }
];

// Sort options
const sortOptions = [
  { value: 'date', label: 'Latest Modified' },
  { value: 'priority', label: 'Highest Priority' }
];

// Priority mapping for sorting (higher number = higher priority)
const priorityOrder = {
  'critical': 4,
  'high': 3,
  'medium': 2,
  'low': 1,
  'default': 0
};

// Watch for changes to the filterStatus prop
watch(() => props.filterStatus, (newStatus) => {
  selectedStatus.value = newStatus;
  fetchAssignedTickets();
});

// Format relative time
const formatRelativeTime = (dateString: string) => {
  const date = new Date(dateString);
  const now = new Date();
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
  
  if (diffInSeconds < 60) {
    return 'just now';
  } else if (diffInSeconds < 3600) {
    const minutes = Math.floor(diffInSeconds / 60);
    return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'} ago`;
  } else if (diffInSeconds < 86400) {
    const hours = Math.floor(diffInSeconds / 3600);
    return `${hours} ${hours === 1 ? 'hour' : 'hours'} ago`;
  } else {
    const days = Math.floor(diffInSeconds / 86400);
    if (days < 7) {
      return `${days} ${days === 1 ? 'day' : 'days'} ago`;
    } else {
      return date.toLocaleDateString('en-US', { 
        month: 'short', 
        day: 'numeric',
        year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
      });
    }
  }
};

// Get class for priority badge
const getPriorityClass = (priority: string) => {
  switch (priority) {
    case 'critical':
      return 'bg-red-900/50 text-red-300';
    case 'high':
      return 'bg-orange-900/50 text-orange-300';
    case 'medium':
      return 'bg-yellow-900/50 text-yellow-300';
    case 'low':
      return 'bg-green-900/50 text-green-300';
    default:
      return 'bg-slate-700 text-slate-300';
  }
};

// Get class for status badge
const getStatusClass = (status: string) => {
  switch (status) {
    case 'open':
      return 'bg-green-900/50 text-green-300';
    case 'in-progress':
      return 'bg-blue-900/50 text-blue-300';
    case 'closed':
      return 'bg-slate-700 text-slate-300';
    default:
      return 'bg-slate-700 text-slate-300';
  }
};

// Get formatted status for display
const getFormattedStatus = (status: string) => {
  switch (status) {
    case 'in-progress':
      return 'In Progress';
    default:
      return status.charAt(0).toUpperCase() + status.slice(1);
  }
};

// Get tickets assigned to the current user
const fetchAssignedTickets = async () => {
  if (!authStore.user) return;
  
  loading.value = true;
  error.value = null;
  
  try {
    // Fetch all tickets
    const allTickets = await ticketService.getTickets();
    
    // Filter tickets assigned to the current user
    let userTickets = allTickets.filter(ticket => 
      ticket.assignee === authStore.user?.uuid || 
      ticket.assignee === authStore.user?.email ||
      ticket.assignee === authStore.user?.name
    );
    
    // Apply status filter if selected
    if (selectedStatus.value) {
      userTickets = userTickets.filter(ticket => ticket.status === selectedStatus.value);
    }
    
    // Sort tickets based on selected sort method
    if (sortBy.value === 'priority') {
      // Sort by priority (highest to lowest)
      userTickets.sort((a, b) => {
        const priorityA = priorityOrder[a.priority as keyof typeof priorityOrder] || priorityOrder.default;
        const priorityB = priorityOrder[b.priority as keyof typeof priorityOrder] || priorityOrder.default;
        
        // If priorities are equal, sort by date (newest first)
        if (priorityB === priorityA) {
          return new Date(b.modified).getTime() - new Date(a.modified).getTime();
        }
        
        return priorityB - priorityA;
      });
    } else {
      // Sort by modified date (newest first)
      userTickets.sort((a, b) => new Date(b.modified).getTime() - new Date(a.modified).getTime());
    }
    
    // Limit the number of tickets shown
    tickets.value = userTickets.slice(0, props.limit);
  } catch (err) {
    console.error('Error fetching assigned tickets:', err);
    error.value = 'Failed to load your assigned tickets';
  } finally {
    loading.value = false;
  }
};

const navigateToTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`);
};

// Get status counts for the tickets
const statusCounts = computed(() => {
  if (!authStore.user) return { total: 0, open: 0, inProgress: 0, closed: 0 };
  
  // This would normally be done on the backend with a dedicated API
  // Here we're just counting the tickets we already have
  return {
    total: tickets.value.length,
    open: tickets.value.filter(t => t.status === 'open').length,
    inProgress: tickets.value.filter(t => t.status === 'in-progress').length,
    closed: tickets.value.filter(t => t.status === 'closed').length
  };
});

onMounted(() => {
  fetchAssignedTickets();
});
</script>

<template>
  <div class="bg-slate-800 rounded-lg overflow-hidden">
    <!-- Header with title and filter -->
    <div class="px-6 py-4 border-b border-slate-700 flex flex-col sm:flex-row sm:justify-between sm:items-center gap-3">
      <div v-if="showTitle" class="text-lg font-medium text-white">Your Assigned Tickets</div>
      
      <div class="flex flex-col sm:flex-row gap-3">
        <!-- Sort dropdown -->
        <div class="relative">
          <select
            v-model="sortBy"
            @change="fetchAssignedTickets"
            class="bg-slate-700 border border-slate-600 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2 text-white"
          >
            <option v-for="option in sortOptions" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>
        
        <!-- Filter dropdown -->
        <div class="relative">
          <select
            v-model="selectedStatus"
            @change="fetchAssignedTickets"
            class="bg-slate-700 border border-slate-600 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2 text-white"
          >
            <option v-for="option in statusOptions" :key="option.value" :value="option.value">
              {{ option.label }}
            </option>
          </select>
        </div>
      </div>
    </div>
    
    <!-- Loading state -->
    <div v-if="loading" class="px-6 py-12 flex justify-center items-center">
      <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
    </div>
    
    <!-- Error state -->
    <div v-else-if="error" class="px-6 py-8 text-center text-red-400">
      {{ error }}
      <button 
        @click="fetchAssignedTickets" 
        class="mt-4 px-4 py-1 bg-slate-700 rounded-lg text-white hover:bg-slate-600 transition-colors"
      >
        Retry
      </button>
    </div>
    
    <!-- Empty state -->
    <div v-else-if="tickets.length === 0" class="px-6 py-8 text-center text-slate-400">
      <div class="flex flex-col items-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mb-4 text-slate-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
        </svg>
        <p>No assigned tickets found</p>
      </div>
    </div>
    
    <!-- Ticket list -->
    <div v-else class="divide-y divide-slate-700">
      <div 
        v-for="ticket in tickets" 
        :key="ticket.id"
        @click="navigateToTicket(ticket.id)"
        class="px-6 py-4 hover:bg-slate-700/50 transition-colors cursor-pointer"
      >
        <div class="flex justify-between items-start">
          <div class="flex-1 min-w-0">
            <h3 class="text-white font-medium truncate">{{ ticket.title }}</h3>
            <div class="flex items-center gap-2 mt-1 text-sm">
              <span class="text-slate-400">#{{ ticket.id }}</span>
              <span class="text-slate-500">•</span>
              <span class="text-slate-400">{{ formatRelativeTime(ticket.modified) }}</span>
            </div>
          </div>
          <div class="flex flex-col items-end gap-2">
            <!-- Priority and status badges -->
            <div class="flex gap-2">
              <span 
                class="px-2 py-1 text-xs rounded-full"
                :class="getPriorityClass(ticket.priority)"
              >
                {{ ticket.priority.charAt(0).toUpperCase() + ticket.priority.slice(1) }}
              </span>
              <span 
                class="px-2 py-1 text-xs rounded-full"
                :class="getStatusClass(ticket.status)"
              >
                {{ getFormattedStatus(ticket.status) }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Footer with view all link -->
    <div v-if="tickets.length > 0" class="px-6 py-3 bg-slate-750 flex justify-center">
      <router-link to="/tickets?assignee=current" class="text-blue-400 hover:text-blue-300 text-sm">
        View all your tickets
      </router-link>
    </div>
  </div>
</template> 