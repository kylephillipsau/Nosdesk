<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';

interface Props {
  ticketStatus?: 'open' | 'in-progress' | 'closed';
}

const props = withDefaults(defineProps<Props>(), {
  ticketStatus: 'closed'
});

interface DayData {
  date: string;
  count: number;
}

const heatmapData = ref<DayData[]>([]);

// Generate last 12 months of dates
const generateDates = () => {
  const dates: DayData[] = [];
  const today = new Date();
  
  for (let i = 365; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    dates.push({
      date: date.toISOString().split('T')[0],
      count: Math.floor(Math.random() * 5) // Placeholder data - replace with real data
    });
  }
  
  return dates;
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
  heatmapData.value = generateDates();
});
</script>

<template>
  <div class="bg-slate-800 rounded-lg p-6">
    <h3 class="text-gray-400 text-sm font-medium mb-4">Activity Heatmap</h3>
    
    <div class="w-full">
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
              :title="`${formatDate(day.date)}: ${day.count} tickets`"
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
  </div>
</template>