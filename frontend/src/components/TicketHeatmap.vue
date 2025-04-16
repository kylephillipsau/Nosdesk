<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { getTickets } from '@/services/ticketService';
import type { Ticket } from '@/services/ticketService';

interface Props {
  ticketStatus?: 'open' | 'in-progress' | 'closed';
}

const props = withDefaults(defineProps<Props>(), {
  ticketStatus: 'closed'
});

interface DayData {
  date: string;
  count: number;
  tickets: { id: number; title: string }[];
}

const heatmapData = ref<DayData[]>([]);
const isLoading = ref(true);
const error = ref<string | null>(null);

// Generate empty data structure for the last 365 days
const generateEmptyDates = () => {
  const dates: DayData[] = [];
  const today = new Date();
  
  for (let i = 365; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    dates.push({
      date: date.toISOString().split('T')[0],
      count: 0,
      tickets: []
    });
  }
  
  return dates;
};

// Fetch ticket data and populate the heatmap
const fetchTicketData = async () => {
  isLoading.value = true;
  error.value = null;
  
  try {
    // Initialize empty dates
    const emptyDates = generateEmptyDates();
    const dateMap = new Map<string, { count: number; tickets: { id: number; title: string }[] }>();
    
    // Create a map of dates for quick lookup
    emptyDates.forEach(day => {
      dateMap.set(day.date, { count: 0, tickets: [] });
    });
    
    // Fetch tickets
    const tickets = await getTickets();
    
    // Filter tickets based on status prop and count them by date
    tickets.forEach(ticket => {
      if (ticket.status === props.ticketStatus) {
        // For closed tickets, use the closed_at date if available, otherwise fall back to modified
        const dateStr = ticket.status === 'closed' && ticket.closed_at 
          ? ticket.closed_at.split('T')[0] 
          : ticket.modified.split('T')[0];
        
        if (dateMap.has(dateStr)) {
          const dayData = dateMap.get(dateStr)!;
          dayData.count++;
          dayData.tickets.push({ id: ticket.id, title: ticket.title });
        }
      }
    });
    
    // Convert map back to array
    heatmapData.value = emptyDates.map(day => ({
      date: day.date,
      count: dateMap.get(day.date)?.count || 0,
      tickets: dateMap.get(day.date)?.tickets || []
    }));
    
  } catch (err: any) {
    console.error('Error fetching ticket data for heatmap:', err);
    error.value = 'Failed to load ticket data. Please try again.';
  } finally {
    isLoading.value = false;
  }
};

// Get color based on activity count using custom green gradient
const getColor = (count: number) => {
  if (count === 0) return '#1D293D'; // Base slate with slight green tint
  if (count <= 1) return '#165142';
  if (count <= 2) return '#0F7947';
  if (count <= 3) return '#08A14C';
  return '#00C950'; // Full intensity
};

// Format date for tooltip
const formatDate = (date: string) => {
  return new Date(date).toLocaleDateString('en-US', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
};

// Create tooltip content with ticket details
const getTooltipContent = (day: DayData) => {
  const dateStr = formatDate(day.date);
  if (day.count === 0) {
    return `${dateStr}: No tickets`;
  }
  
  const ticketWord = day.count === 1 ? 'ticket' : 'tickets';
  let tooltip = `${dateStr}: ${day.count} ${ticketWord}`;
  
  // Add ticket titles if available (limit to first 5)
  if (day.tickets.length > 0) {
    tooltip += '\n\nTickets:';
    const displayTickets = day.tickets.slice(0, 5);
    displayTickets.forEach(ticket => {
      tooltip += `\n• #${ticket.id}: ${ticket.title}`;
    });
    
    if (day.tickets.length > 5) {
      tooltip += `\n• ...and ${day.tickets.length - 5} more`;
    }
  }
  
  return tooltip;
};

// Group data by week for display
const weeklyData = computed(() => {
  const weeks: DayData[][] = [];
  let currentWeek: DayData[] = [];
  
  heatmapData.value.forEach((day, index) => {
    const date = new Date(day.date);
    currentWeek.push(day);
    
    if (date.getDay() === 6 || index === heatmapData.value.length - 1) {
      weeks.push(currentWeek);
      currentWeek = [];
    }
  });
  
  return weeks;
});

onMounted(() => {
  fetchTicketData();
});
</script>

<template>
  <div class="bg-slate-800 rounded-lg p-6">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-gray-400 text-sm font-medium">{{ props.ticketStatus === 'closed' ? 'Closed Tickets' : 'Ticket Activity' }} Heatmap</h3>
      <button 
        @click="fetchTicketData" 
        class="text-xs text-gray-400 hover:text-white transition-colors"
        :disabled="isLoading"
      >
        <span v-if="isLoading">Loading...</span>
        <span v-else>Refresh</span>
      </button>
    </div>
    
    <div v-if="error" class="text-red-400 text-sm mb-4">
      {{ error }}
    </div>
    
    <div class="w-full" v-if="!isLoading && !error">
      <div class="flex gap-[1px] w-full">
        <!-- Days of week labels -->
        <div class="flex flex-col gap-1 text-xs text-gray-400 mr-2">
          <span class="h-3 flex items-center">Sun</span>
          <span class="h-3 flex items-center">Mon</span>
          <span class="h-3 flex items-center">Tue</span>
          <span class="h-3 flex items-center">Wed</span>
          <span class="h-3 flex items-center">Thu</span>
          <span class="h-3 flex items-center">Fri</span>
          <span class="h-3 flex items-center">Sat</span>
        </div>
        
        <!-- Heatmap grid -->
        <div class="flex flex-1 gap-0.5">
          <div 
            v-for="(week, weekIndex) in weeklyData" 
            :key="weekIndex" 
            class="flex flex-col flex-1 gap-0.5"
          >
            <div
              v-for="day in week"
              :key="day.date"
              class="h-3.5 rounded-sm cursor-pointer transition-colors duration-200"
              :style="{ backgroundColor: getColor(day.count) }"
              :title="getTooltipContent(day)"
            />
          </div>
        </div>
      </div>
      
      <!-- Legend -->
      <div class="flex items-center gap-2 mt-4 text-xs text-gray-400">
        <span>Less</span>
        <div class="flex gap-1">
          <div
            v-for="i in 5"
            :key="i"
            class="w-3 h-3 rounded-sm"
            :style="{ backgroundColor: getColor(i - 1) }"
          />
        </div>
        <span>More</span>
      </div>
    </div>
    
    <div v-else-if="isLoading" class="flex justify-center items-center h-32 text-gray-400">
      Loading ticket data...
    </div>
  </div>
</template>