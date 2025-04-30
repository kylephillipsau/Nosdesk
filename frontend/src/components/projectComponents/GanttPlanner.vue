<!-- GanttPlanner.vue -->
<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { projectService } from '@/services/projectService'
import ticketService from '@/services/ticketService'
import UserAvatar from '@/components/UserAvatar.vue'
import StatusBadge from '@/components/StatusBadge.vue'

const props = defineProps<{
  projectId: number;
  tickets: any[]; // Using any[] until we have a proper type
}>()

const router = useRouter()
const isLoading = ref(false)
const error = ref<string | null>(null)
const ganttTickets = ref<any[]>([])
const timeScale = ref<'day' | 'week' | 'month'>('week')
const today = new Date()
const startDate = ref(new Date(today.getFullYear(), today.getMonth(), 1)) // First day of current month
const endDate = ref(new Date(today.getFullYear(), today.getMonth() + 3, 0)) // Last day of +3 months

// Convert tickets to Gantt format
const processTickets = () => {
  if (!props.tickets || props.tickets.length === 0) {
    ganttTickets.value = []
    return
  }

  ganttTickets.value = props.tickets.map(ticket => {
    // Default start date (today if none available)
    let start = ticket.createdAt ? new Date(ticket.createdAt) : new Date()
    
    // Default end date (7 days from start if none available)
    let end
    if (ticket.dueDate) {
      end = new Date(ticket.dueDate)
    } else {
      end = new Date(start)
      end.setDate(end.getDate() + 7) // Default 7 day span
    }

    // Define color based on priority
    let color
    switch (ticket.priority) {
      case 'high':
        color = '#ef4444' // Red
        break
      case 'medium':
        color = '#f59e0b' // Amber
        break
      case 'low':
        color = '#3b82f6' // Blue
        break
      default:
        color = '#6b7280' // Gray
    }

    return {
      id: ticket.id,
      title: ticket.title,
      start,
      end,
      color,
      progress: ticket.status === 'closed' ? 100 : 
                ticket.status === 'in-progress' ? 50 : 0,
      status: ticket.status,
      assignee: ticket.assignee,
      priority: ticket.priority
    }
  })
}

// Initialize data when tickets prop changes
watch(() => props.tickets, () => {
  processTickets()
}, { immediate: true })

// Date formatting helpers
const formatDate = (date: Date) => {
  return date.toLocaleDateString('en-US', { 
    month: 'short', 
    day: 'numeric'
  })
}

const formatHeaderDate = (date: Date) => {
  const month = date.toLocaleDateString('en-US', { month: 'short' })
  const day = date.getDate()
  return `${month} ${day}`
}

// Generate time units for the header based on timeScale
const timeUnits = computed(() => {
  const units = []
  let current = new Date(startDate.value)
  
  while (current <= endDate.value) {
    units.push(new Date(current))
    
    // Increment based on timeScale
    if (timeScale.value === 'day') {
      current.setDate(current.getDate() + 1)
    } else if (timeScale.value === 'week') {
      current.setDate(current.getDate() + 7)
    } else {
      current.setMonth(current.getMonth() + 1)
    }
  }
  
  return units
})

// Helper to calculate position and width of a task bar
const getTaskBarStyle = (task: any) => {
  // Calculate total days in the chart
  const totalDays = Math.ceil((endDate.value.getTime() - startDate.value.getTime()) / (1000 * 60 * 60 * 24))
  
  // Calculate task start position
  const taskStartDays = Math.max(0, Math.ceil((task.start.getTime() - startDate.value.getTime()) / (1000 * 60 * 60 * 24)))
  const startPercent = (taskStartDays / totalDays) * 100
  
  // Calculate task duration
  const taskDurationDays = Math.ceil((task.end.getTime() - task.start.getTime()) / (1000 * 60 * 60 * 24))
  const widthPercent = (taskDurationDays / totalDays) * 100
  
  return {
    left: `${startPercent}%`,
    width: `${widthPercent}%`,
    backgroundColor: task.color,
    opacity: task.status === 'closed' ? 0.7 : 1
  }
}

// Navigate to ticket detail
const openTicket = (ticketId: number) => {
  router.push(`/tickets/${ticketId}`)
}

// Change time scale
const setTimeScale = (scale: 'day' | 'week' | 'month') => {
  timeScale.value = scale
}

// Adjust date range
const zoomIn = () => {
  const currentRange = endDate.value.getTime() - startDate.value.getTime()
  const newRange = currentRange * 0.7
  
  // Recalculate end date based on the new range
  const newEndDate = new Date(startDate.value.getTime() + newRange)
  endDate.value = newEndDate
}

const zoomOut = () => {
  const currentRange = endDate.value.getTime() - startDate.value.getTime()
  const newRange = currentRange * 1.3
  
  // Recalculate end date based on the new range
  const newEndDate = new Date(startDate.value.getTime() + newRange)
  endDate.value = newEndDate
}

const moveRangeLeft = () => {
  const rangeSize = endDate.value.getTime() - startDate.value.getTime()
  const moveAmount = rangeSize * 0.25 // Move 25% of the range
  
  startDate.value = new Date(startDate.value.getTime() - moveAmount)
  endDate.value = new Date(endDate.value.getTime() - moveAmount)
}

const moveRangeRight = () => {
  const rangeSize = endDate.value.getTime() - startDate.value.getTime()
  const moveAmount = rangeSize * 0.25 // Move 25% of the range
  
  startDate.value = new Date(startDate.value.getTime() + moveAmount)
  endDate.value = new Date(endDate.value.getTime() + moveAmount)
}

// Reset to today
const resetView = () => {
  startDate.value = new Date(today.getFullYear(), today.getMonth(), 1)
  endDate.value = new Date(today.getFullYear(), today.getMonth() + 3, 0)
}

// Helper function to check if a date is the same day as today
const isSameDay = (date: Date, today: Date) => {
  return date.toDateString() === today.toDateString()
}
</script>

<template>
  <div class="h-full flex flex-col relative">
    <!-- Error message -->
    <div v-if="error" class="bg-red-500/20 border border-red-500/50 text-red-200 px-4 py-3 rounded-lg mb-4">
      {{ error }}
    </div>
    
    <!-- Loading state -->
    <div v-if="isLoading" class="flex justify-center items-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
    </div>
    
    <div v-else class="flex-1 flex flex-col gap-2">
      <!-- Controls -->
      <div class="flex items-center justify-between mb-2 bg-slate-800 p-2 rounded-lg">
        <div class="flex items-center gap-2">
          <button 
            @click="setTimeScale('day')" 
            class="px-2 py-0.5 text-xs rounded-md transition-colors"
            :class="timeScale === 'day' ? 'bg-blue-600 text-white' : 'bg-slate-700 text-slate-300 hover:bg-slate-600'"
          >
            Day
          </button>
          <button 
            @click="setTimeScale('week')" 
            class="px-2 py-0.5 text-xs rounded-md transition-colors"
            :class="timeScale === 'week' ? 'bg-blue-600 text-white' : 'bg-slate-700 text-slate-300 hover:bg-slate-600'"
          >
            Week
          </button>
          <button 
            @click="setTimeScale('month')" 
            class="px-2 py-0.5 text-xs rounded-md transition-colors"
            :class="timeScale === 'month' ? 'bg-blue-600 text-white' : 'bg-slate-700 text-slate-300 hover:bg-slate-600'"
          >
            Month
          </button>
        </div>
        
        <div class="flex items-center space-x-1 gap-2">
          <button 
            @click="moveRangeLeft" 
            class="p-1 bg-slate-700 text-slate-300 rounded-md hover:bg-slate-600 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          
          <button 
            @click="resetView" 
            class="px-1.5 py-0.5 bg-slate-700 text-slate-300 rounded-md hover:bg-slate-600 transition-colors text-xs"
          >
            Today
          </button>
          
          <button 
            @click="moveRangeRight" 
            class="p-1 bg-slate-700 text-slate-300 rounded-md hover:bg-slate-600 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
          
          <div class="h-4 border-r border-slate-600 mx-1"></div>
          
          <button 
            @click="zoomIn" 
            class="p-1 bg-slate-700 text-slate-300 rounded-md hover:bg-slate-600 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" />
            </svg>
          </button>
          
          <button 
            @click="zoomOut" 
            class="p-1 bg-slate-700 text-slate-300 rounded-md hover:bg-slate-600 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7" />
            </svg>
          </button>
        </div>
        
        <div class="text-xs text-slate-400">
          {{ formatDate(startDate) }} - {{ formatDate(endDate) }}
        </div>
      </div>
      
      <!-- Gantt Chart -->
      <div class="flex-1 overflow-x-auto bg-slate-800 rounded-lg p-2">
        <div v-if="ganttTickets.length === 0" class="flex justify-center items-center h-full">
          <div class="text-slate-400">No tickets available for Gantt view</div>
        </div>
        
        <div v-else class="gantt-chart">
          <!-- Header with timeline -->
          <div class="gantt-header">
            <!-- Left side header (ticket info) -->
            <div class="gantt-header-left">
              <div class="gantt-header-item flex items-center">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-slate-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                </svg>
                <span class="text-xs font-medium text-slate-300">Ticket Details</span>
              </div>
            </div>
            
            <!-- Right side header (timeline) -->
            <div class="gantt-header-timeline">
              <div 
                v-for="(unit, index) in timeUnits" 
                :key="index" 
                class="gantt-timeline-unit"
              >
                <span 
                  :class="{'text-blue-400 font-medium': isSameDay(unit, today)}"
                  class="whitespace-nowrap"
                >
                  {{ formatHeaderDate(unit) }}
                </span>
              </div>
            </div>
          </div>
          
          <!-- Gantt Body -->
          <div class="gantt-body">
            <div 
              v-for="ticket in ganttTickets" 
              :key="ticket.id" 
              class="gantt-row"
              @click="openTicket(ticket.id)"
            >
              <!-- Ticket info -->
              <div class="gantt-row-left">
                <!-- First row: Ticket title with ID -->
                <div class="flex items-center gap-2 w-full overflow-hidden">
                  <span class="shrink-0 text-xs text-slate-400 font-mono">#{{ ticket.id }}</span>
                  <span class="ticket-title font-medium truncate text-sm text-slate-100">{{ ticket.title }}</span>
                </div>
                
                <!-- Second row: Status badges and assignee -->
                <div class="flex flex-wrap items-center gap-1 mt-1">
                  <!-- Status badge -->
                  <span
                    class="text-[10px] px-1.5 py-0.5 rounded-sm font-medium uppercase tracking-wide"
                    :class="{
                      'bg-blue-500/20 text-blue-400': ticket.status === 'in-progress',
                      'bg-green-500/20 text-green-400': ticket.status === 'closed',
                      'bg-yellow-500/20 text-yellow-400': ticket.status === 'open'
                    }"
                  >
                    {{ ticket.status }}
                  </span>
                  
                  <!-- Priority badge -->
                  <span
                    class="text-[10px] px-1.5 py-0.5 rounded-sm font-medium uppercase tracking-wide"
                    :class="{
                      'bg-red-500/20 text-red-400': ticket.priority === 'high',
                      'bg-yellow-500/20 text-yellow-400': ticket.priority === 'medium',
                      'bg-blue-500/20 text-blue-400': ticket.priority === 'low'
                    }"
                  >
                    {{ ticket.priority }}
                  </span>
                  
                  <!-- Assignee avatar -->
                  <div class="ml-auto" @click.stop>
                    <UserAvatar 
                      v-if="ticket.assignee" 
                      :name="ticket.assignee" 
                      size="xs" 
                      :showName="false"
                      :clickable="true"
                    />
                  </div>
                </div>
              </div>
              
              <!-- Timeline with task bar -->
              <div class="gantt-row-timeline">
                <!-- Task bar -->
                <div 
                  class="gantt-task-bar relative" 
                  :style="getTaskBarStyle(ticket)"
                >
                  <!-- Progress bar -->
                  <div 
                    class="gantt-task-progress" 
                    :style="{ width: `${ticket.progress}%` }"
                  ></div>
                  
                  <!-- Task label (shown on wider bars) -->
                  <div class="absolute inset-0 flex items-center px-2 overflow-hidden" 
                       v-if="parseFloat(getTaskBarStyle(ticket).width.replace('%', '')) > 8">
                    <span class="text-xs text-white truncate drop-shadow-sm">
                      {{ ticket.title }}
                    </span>
                  </div>
                </div>
                
                <!-- Today marker (vertical line) -->
                <div 
                  v-if="today >= startDate && today <= endDate"
                  class="gantt-today-marker"
                  :style="{
                    left: `${((today.getTime() - startDate.getTime()) / 
                           (endDate.getTime() - startDate.getTime())) * 100}%`
                  }"
                >
                  <div class="today-marker-label">Today</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.gantt-chart {
  min-width: 900px;
  min-height: 300px;
  position: relative;
}

.gantt-header {
  display: flex;
  border-bottom: 1px solid #334155; /* slate-700 */
  height: 36px;
}

.gantt-header-left {
  width: 240px;
  min-width: 240px;
  border-right: 1px solid #334155; /* slate-700 */
  padding: 0 12px;
  display: flex;
  align-items: center;
  position: sticky;
  left: 0;
  background-color: #1e293b; /* slate-800 */
  z-index: 10;
}

.gantt-header-timeline {
  flex: 1;
  display: flex;
  overflow-x: hidden;
}

.gantt-timeline-unit {
  flex: 1;
  min-width: 80px;
  padding: 8px 4px;
  text-align: center;
  font-size: 11px;
  color: #94a3b8; /* slate-400 */
  border-right: 1px solid #334155; /* slate-700 */
}

.gantt-body {
  position: relative;
}

.gantt-row {
  display: flex;
  height: 52px; /* Slightly increased from 48px to accommodate two rows */
  border-bottom: 1px solid #334155; /* slate-700 */
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.gantt-row:hover {
  background-color: #334155; /* slate-700 */
}

.gantt-row-left {
  width: 240px;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 4px 12px;
  border-right: 1px solid #334155; /* slate-700 */
  position: sticky;
  left: 0;
  z-index: 5;
  transition: background-color 0.15s ease;
}

.gantt-row:hover .gantt-row-left {
  background-color: #334155; /* slate-700 */
}

.gantt-row-title {
  color: #e2e8f0; /* slate-200 */
  font-size: 13px;
  line-height: 1.2;
}

.gantt-task-bar {
  position: absolute;
  height: 18px; /* Reduced from 22px */
  top: 50%;
  transform: translateY(-50%);
  border-radius: 3px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.gantt-task-bar:hover {
  opacity: 0.9;
}

.gantt-task-progress {
  height: 100%;
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.gantt-today-marker {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background-color: #ef4444; /* red-500 */
  z-index: 1;
}

.today-marker-label {
  position: absolute;
  top: -1px;
  left: 50%;
  transform: translateX(-50%);
  background-color: #ef4444; /* red-500 */
  color: white;
  font-size: 10px;
  padding: 1px 3px;
  border-radius: 0 0 3px 3px;
  white-space: nowrap;
  font-weight: 500;
}

/* Add subtle grid lines for better visualization */
.gantt-row-timeline {
  flex: 1;
  position: relative;
  overflow: hidden;
  background-image: repeating-linear-gradient(
    90deg,
    transparent,
    transparent 99px,
    rgba(71, 85, 105, 0.2) 99px,
    rgba(71, 85, 105, 0.2) 100px
  );
}

/* Add alternating row colors for better readability */
.gantt-row:nth-child(even) {
  background-color: rgba(51, 65, 85, 0.15);
}

.gantt-row:nth-child(even) .gantt-row-left {
  background-color: rgba(30, 41, 59, 0.95);
}

.gantt-row:nth-child(even):hover,
.gantt-row:nth-child(even):hover .gantt-row-left {
  background-color: #334155; /* slate-700 */
}
</style> 