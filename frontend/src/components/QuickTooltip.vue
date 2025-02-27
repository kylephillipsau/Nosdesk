<script setup lang="ts">
import { ref, onMounted } from 'vue'
import UserAvatar from './UserAvatar.vue'

interface TicketDetails {
  title: string
  requester?: string
  assignee?: string
  status?: string
  created?: string
}

const props = defineProps<{
  text: string
  details?: TicketDetails
  position?: 'top' | 'bottom' | 'left' | 'right'
  delay?: number
  disabled?: boolean
}>()

const container = ref<HTMLElement | null>(null)
const tooltipTop = ref(0)

const updatePosition = () => {
  if (container.value) {
    const rect = container.value.getBoundingClientRect()
    tooltipTop.value = rect.top + (rect.height / 2)
  }
}
</script>

<template>
  <div 
    class="relative flex-1 min-w-0" 
    ref="container"
    @mouseenter="updatePosition"
    @mouseover="updatePosition"
  >
    <slot />
    <div 
      v-if="!disabled"
      class="absolute invisible opacity-0 group-hover:visible group-hover:opacity-100
             bg-gray-900 text-white text-xs px-3 py-2 rounded shadow-lg
             pointer-events-none z-[9999] w-[240px]"
      :style="{
        transitionDelay: `${delay || 0}ms`,
        transitionProperty: 'opacity',
        transitionDuration: '50ms',
        position: 'fixed',
        left: 'calc(256px + 0.5rem)', // 256px is the navbar width (w-64 = 16rem = 256px)
        top: `${tooltipTop}px`,
        transform: 'translateY(-50%)'
      }"
    >
      <!-- Arrow pointing left -->
      <div 
        class="absolute -left-2 top-1/2 -translate-y-1/2 w-0 h-0 
               border-t-[6px] border-t-transparent
               border-r-[8px] border-r-gray-900
               border-b-[6px] border-b-transparent"
      ></div>

      <div class="flex flex-col gap-1">
        <div class="font-medium">{{ text }}</div>
        <div v-if="details" class="text-gray-400 flex flex-col gap-2 mt-1">
          <div v-if="details.status" class="flex items-center gap-2">
            <span class="text-gray-500">Status:</span>
            <span>{{ details.status }}</span>
          </div>
          <div v-if="details.requester || details.assignee" class="flex flex-col gap-1.5">
            <div v-if="details.requester" class="flex items-center gap-2">
              <UserAvatar :name="details.requester" :showName="false" size="xs" />
              <span class="truncate">{{ details.requester }}</span>
            </div>
            <div v-if="details.assignee" class="flex items-center gap-2">
              <UserAvatar :name="details.assignee" :showName="false" size="xs" />
              <span class="truncate">{{ details.assignee }}</span>
            </div>
          </div>
          <div v-if="details.created" class="text-[11px] text-gray-500">
            {{ details.created }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.group\/tooltip {
  isolation: isolate;
}
</style> 