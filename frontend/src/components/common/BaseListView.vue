<script setup lang="ts">
import { computed } from 'vue'
import ListHeader from './ListHeader.vue'

const props = defineProps<{
  title: string
  searchQuery: string
  isLoading: boolean
  isEmpty: boolean
  addButtonText?: string
  showAddButton?: boolean
  emptyMessage?: string
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'add': []
}>()

const defaultEmptyMessage = computed(() => {
  return `No ${props.title.toLowerCase()} found.`
})
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-white p-6 flex justify-center">
    <div class="flex flex-col gap-4 max-w-7xl w-full">
      <ListHeader 
        :title="title"
        :search-query="searchQuery"
        :add-button-text="addButtonText"
        :show-add-button="showAddButton"
        @update:search-query="(value) => emit('update:searchQuery', value)"
        @add="emit('add')"
      />

      <div v-if="isLoading" class="flex justify-center items-center h-64">
        <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
      </div>

      <div v-else-if="isEmpty" class="p-8 text-center text-gray-400 bg-gray-800 rounded-lg">
        {{ emptyMessage || defaultEmptyMessage }}
      </div>

      <div v-else class="bg-gray-800 rounded-lg overflow-hidden">
        <slot></slot>
      </div>
    </div>
  </div>
</template> 