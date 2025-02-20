<!-- components/CustomDropdown.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  value: string
  options: { value: string; label: string }[]
  type: 'status' | 'priority'
}>()

const emit = defineEmits<{
  (e: 'update:value', value: string): void
}>()

const isOpen = ref(false)

const selectedOption = computed(() =>
  props.options.find(option => option.value === props.value)
)

const toggleDropdown = () => {
  isOpen.value = !isOpen.value
}

const selectOption = (option: { value: string; label: string }) => {
  emit('update:value', option.value)
  isOpen.value = false
}

// Close dropdown when clicking outside
const dropdownRef = ref<HTMLElement | null>(null)
const handleClickOutside = (event: MouseEvent) => {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

// Function to determine the color and shadow based on the status or priority value
const getColorAndShadow = (value: string | undefined) => {
  if (!value) return 'bg-gray-500 text-gray-100 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]'

  if (props.type === 'status') {
    switch (value) {
      case 'open':
        return 'bg-yellow-500 text-yellow-100 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]'
      case 'in-progress':
        return 'bg-blue-500 text-blue-100 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]'
      case 'closed':
        return 'bg-green-500 text-green-100 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]'
      default:
        return 'bg-gray-500 text-gray-100 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]'
    }
  } else if (props.type === 'priority') {
    switch (value) {
      case 'low':
        return 'text-green-400'
      case 'medium':
        return 'text-yellow-400'
      case 'high':
        return 'text-red-400'
      default:
        return 'text-gray-400'
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="relative" ref="dropdownRef">
    <!-- Dropdown trigger -->
    <button type="button" @click="toggleDropdown"
      :class="`w-full px-2 py-1 rounded-xl flex items-center justify-between transition-all hover:bg-gray-600 ${getColorAndShadow(value)}`">
      <span class="">{{ selectedOption?.label || 'Select an option' }}</span>
      <svg class="w-4 h-4 transition-transform duration-200" :class="{ 'rotate-180': isOpen }" fill="none"
        stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Dropdown menu -->
    <div v-if="isOpen" class="absolute z-10 w-full mt-1 bg-gray-700 rounded-xl shadow-lg overflow-hidden">
      <ul class="py-1">
        <li v-for="option in options" :key="option.value" @click="selectOption(option)"
          :class="`px-3 py-2 text-gray-200 hover:bg-gray-600 cursor-pointer transition-all flex items-center gap-2 ${option.value && value && option.value === value ? getColorAndShadow(value) : ''}`">
          <span v-if="props.type === 'status'"
            class="inline-block w-5 h-5 rounded-full mr-3 shadow-[inset_0_1px_3px_rgba(0,0,0,0.5)]"
            :class="getColorAndShadow(option.value)?.split(' ')?.[0] || 'bg-gray-500'"></span>
          <span
            :class="{ 'text-gray-200': props.type === 'status', 'text-gray-400': props.type === 'priority' && option.value !== value }">{{
              option.label }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>