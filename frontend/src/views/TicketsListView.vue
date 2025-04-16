// views/TicketsListView.vue
<script setup lang="ts">
import ticketService from "@/services/ticketService";
import { ref, onMounted, computed, watch } from "vue";
import { useRouter } from "vue-router";
import BaseListView from "@/components/common/BaseListView.vue";
import UserAvatar from "@/components/UserAvatar.vue";
import StatusBadge from "@/components/StatusBadge.vue";
import type { Ticket, PaginatedResponse } from "@/services/ticketService";

const tickets = ref<Ticket[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedTickets = ref<number[]>([]);
const lastSelectedTicketId = ref<number | null>(null);

// Sorting state
const sortField = ref<string>("id");
const sortDirection = ref<"asc" | "desc">("asc");

// Search and filter state
const searchQuery = ref("");
const statusFilter = ref<string>("all");
const priorityFilter = ref<string>("all");

// Pagination state
const currentPage = ref(1);
const pageSize = ref(10);
const pageSizeOptions = [10, 25, 50, 100];
const totalItems = ref(0);
const totalPages = ref(1);

// Function to format date in locale format
const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString);
    // Check if date is valid
    if (isNaN(date.getTime())) {
      return dateString;
    }

    // Format date in user's locale with date only (no time)
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch (error) {
    console.error("Error formatting date:", error);
    return dateString;
  }
};

// Fetch tickets from API with pagination
const fetchTickets = async () => {
  loading.value = true;
  error.value = null;

  try {
    // For server-side pagination
    const response = await ticketService.getPaginatedTickets({
      page: currentPage.value,
      pageSize: pageSize.value,
      sortField: sortField.value,
      sortDirection: sortDirection.value,
      search: searchQuery.value,
      status: statusFilter.value !== 'all' ? statusFilter.value : undefined,
      priority: priorityFilter.value !== 'all' ? priorityFilter.value : undefined
    });
    
    tickets.value = response.data;
    totalItems.value = response.total;
    totalPages.value = response.totalPages;
  } catch (err) {
    console.error("Failed to fetch tickets:", err);
    error.value = "Failed to load tickets. Please try again later.";
  } finally {
    loading.value = false;
  }
};

// Load tickets when component mounts
onMounted(() => {
  fetchTickets();
});

// Watch for changes in pagination, sorting, or filtering to refetch data
watch(
  [currentPage, pageSize, sortField, sortDirection, searchQuery, statusFilter, priorityFilter],
  () => {
    fetchTickets();
  }
);

// Get unique status values from tickets
const availableStatuses = computed(() => {
  if (!tickets.value.length) return [];
  const statuses = new Set(tickets.value.map((ticket) => ticket.status));
  return Array.from(statuses) as string[];
});

// Get unique priority values from tickets
const availablePriorities = computed(() => {
  if (!tickets.value.length) return [];
  const priorities = new Set(tickets.value.map((ticket) => ticket.priority));
  return Array.from(priorities) as string[];
});

// Prepare filter options for BaseListView
const filterOptions = computed(() => {
  return [
    {
      name: 'status',
      value: statusFilter.value,
      options: [
        { value: 'all', label: 'All Statuses' },
        ...availableStatuses.value.map(status => ({ 
          value: status, 
          label: status.charAt(0).toUpperCase() + status.slice(1) 
        }))
      ],
      width: 'w-[120px]'
    },
    {
      name: 'priority',
      value: priorityFilter.value,
      options: [
        { value: 'all', label: 'All Priorities' },
        ...availablePriorities.value.map(priority => ({ 
          value: priority, 
          label: priority.charAt(0).toUpperCase() + priority.slice(1) 
        }))
      ],
      width: 'w-[120px]'
    }
  ];
});

// Reset all filters
const resetFilters = () => {
  searchQuery.value = "";
  statusFilter.value = "all";
  priorityFilter.value = "all";
  currentPage.value = 1;
  fetchTickets();
};

// Handle filter updates from BaseListView
const handleFilterUpdate = (name: string, value: string) => {
  if (name === 'status') {
    statusFilter.value = value;
  } else if (name === 'priority') {
    priorityFilter.value = value;
  }
  currentPage.value = 1; // Reset to first page when filters change
};

// Handle sort update from BaseListView
const handleSortUpdate = (field: string, direction: 'asc' | 'desc') => {
  sortField.value = field;
  sortDirection.value = direction;
  currentPage.value = 1; // Reset to first page when sort changes
};

const toggleSelection = (event: Event, ticketIdStr: string) => {
  event.stopPropagation();
  const ticketId = parseInt(ticketIdStr, 10);

  // Handle shift key for range selection
  if (
    event instanceof MouseEvent &&
    event.shiftKey &&
    lastSelectedTicketId.value !== null
  ) {
    const currentIndex = tickets.value.findIndex(
      (ticket) => ticket.id === ticketId
    );
    const lastIndex = tickets.value.findIndex(
      (ticket) => ticket.id === lastSelectedTicketId.value
    );

    if (currentIndex !== -1 && lastIndex !== -1) {
      const startIndex = Math.min(currentIndex, lastIndex);
      const endIndex = Math.max(currentIndex, lastIndex);

      const ticketsToSelect = tickets.value
        .slice(startIndex, endIndex + 1)
        .map((ticket) => ticket.id);

      // Add all tickets in range to selection if they're not already selected
      ticketsToSelect.forEach((id) => {
        if (!selectedTickets.value.includes(id)) {
          selectedTickets.value.push(id);
        }
      });
    }
  } 
  // Handle Ctrl/Cmd key for toggling individual items without affecting others
  else if (event instanceof MouseEvent && (event.ctrlKey || event.metaKey)) {
    const index = selectedTickets.value.indexOf(ticketId);
    if (index === -1) {
      selectedTickets.value.push(ticketId);
    } else {
      selectedTickets.value.splice(index, 1);
    }
    
    // Update last selected ticket
    lastSelectedTicketId.value = ticketId;
  }
  // Regular click - toggle the selection without clearing others
  else {
    const index = selectedTickets.value.indexOf(ticketId);
    if (index === -1) {
      // Add to selection without clearing others
      selectedTickets.value.push(ticketId);
    } else {
      // Remove from selection
      selectedTickets.value.splice(index, 1);
    }

    // Update last selected ticket
    lastSelectedTicketId.value = ticketId;
  }
};

const toggleAllTickets = (event: Event) => {
  event.stopPropagation();
  const checkbox = event.target as HTMLInputElement;
  
  // If we're checking the box, select all visible tickets
  if (checkbox.checked) {
    selectedTickets.value = tickets.value.map((ticket) => ticket.id);
  } 
  // If unchecking, clear all selections
  else {
    selectedTickets.value = [];
  }
  
  // Reset last selected ticket
  lastSelectedTicketId.value = null;
};

const router = useRouter();

const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`);
};

// Define columns for the table
const columns = [
  { field: 'id', label: 'ID', width: 'w-20 flex-shrink-0' },
  { field: 'title', label: 'Title', width: 'flex-1 min-w-0' },
  { field: 'status', label: 'Status', width: 'w-24 flex-shrink-0' },
  { field: 'priority', label: 'Priority', width: 'w-24 flex-shrink-0' },
  { field: 'created', label: 'Created', width: 'w-32 flex-shrink-0' },
  { field: 'requester', label: 'Requester', width: 'w-32 flex-shrink-0' },
  { field: 'assignee', label: 'Assignee', width: 'w-32 flex-shrink-0' }
];

// Handle page change
const handlePageChange = (page: number) => {
  currentPage.value = page;
};

// Handle page size change
const handlePageSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1; // Reset to first page when changing page size
};
</script>

<template>
  <BaseListView
    title="Tickets"
    :search-query="searchQuery"
    :is-loading="loading"
    :is-empty="tickets.length === 0 && !loading"
    :error="error"
    :filters="filterOptions"
    :results-count="totalItems"
    :selected-items="selectedTickets.map(id => id.toString())"
    :visible-items="tickets"
    :item-id-field="'id'"
    :enable-selection="true"
    :sort-field="sortField"
    :sort-direction="sortDirection"
    :columns="columns"
    :current-page="currentPage"
    :total-pages="totalPages"
    :page-size="pageSize"
    :page-size-options="pageSizeOptions"
    @update:search-query="value => searchQuery = value"
    @update:filter="handleFilterUpdate"
    @update:sort="handleSortUpdate"
    @toggle-selection="toggleSelection"
    @toggle-all="toggleAllTickets"
    @reset-filters="resetFilters"
    @retry="fetchTickets"
    @update:current-page="handlePageChange"
    @update:page-size="handlePageSizeChange"
  >
    <!-- Desktop Table View -->
    <template #default>
      <div class="min-w-[960px]">
        <div
          v-for="ticket in tickets"
          :key="ticket.id"
          class="flex border-b border-slate-800 text-sm text-gray-200 hover:bg-slate-800/50 transition-colors cursor-pointer gap-2"
          @click="openTicket(ticket.id)"
        >
          <div class="flex items-center p-3 w-10 flex-shrink-0">
            <input
              type="checkbox"
              class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
              :checked="selectedTickets.includes(ticket.id)"
              @click.stop="(event) => toggleSelection(event, ticket.id.toString())"
            />
          </div>
          <div class="flex items-center p-3 w-20 flex-shrink-0">
            #{{ ticket.id }}
          </div>
          <div class="flex items-center p-3 flex-1 min-w-0">
            <div class="truncate">{{ ticket.title }}</div>
          </div>
          <div class="flex items-center w-24 flex-shrink-0 whitespace-nowrap">
            <StatusBadge type="status" :value="ticket.status" :short="true" />
          </div>
          <div class="flex items-center p-2 w-24 flex-shrink-0">
            <StatusBadge type="priority" :value="ticket.priority" :short="true" />
          </div>
          <div class="flex items-center p-3 w-32 flex-shrink-0">
            {{ formatDate(ticket.created) }}
          </div>
          <div class="flex items-center p-1 w-32 flex-shrink-0">
            <UserAvatar :name="ticket.requester" size="sm" />
          </div>
          <div class="flex items-center p-1 w-32 flex-shrink-0">
            <UserAvatar :name="ticket.assignee" size="sm" />
          </div>
        </div>
      </div>
    </template>

    <!-- Mobile Card View -->
    <template #mobile-view>
      <div class="space-y-2 p-2">
        <div
          v-for="ticket in tickets"
          :key="ticket.id"
          @click="openTicket(ticket.id)"
          class="bg-slate-800 rounded-lg p-3 hover:bg-slate-700/50 transition-colors cursor-pointer"
        >
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <input
                type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="selectedTickets.includes(ticket.id)"
                @click.stop="(event) => toggleSelection(event, ticket.id.toString())"
              />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <div class="font-medium truncate">{{ ticket.title }}</div>
                <div class="text-xs text-gray-400 ml-2">#{{ ticket.id }}</div>
              </div>
              <div class="mt-2 flex flex-wrap gap-2 text-xs">
                <StatusBadge type="status" :value="ticket.status" :short="true" />
                <StatusBadge type="priority" :value="ticket.priority" :short="true" />
              </div>
              <div class="mt-2 flex items-center gap-2 text-xs">
                <div class="flex items-center gap-1">
                  <UserAvatar :name="ticket.requester" size="xs" />
                  <span class="text-gray-400">Requester</span>
                </div>
                <div class="flex items-center gap-1">
                  <UserAvatar :name="ticket.assignee" size="xs" />
                  <span class="text-gray-400">Assignee</span>
                </div>
              </div>
              <div class="mt-2 text-xs text-gray-400">
                Created: {{ formatDate(ticket.created) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </BaseListView>
</template>

<style scoped>
/* Optional: Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}
</style>