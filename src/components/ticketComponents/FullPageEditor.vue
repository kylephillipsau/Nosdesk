<!-- FullPageEditor.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import Modal from '@/components/Modal.vue'

const props = defineProps<{
  show: boolean;
  initialContent: string;
}>()

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', content: string): void;
}>()

const content = ref(props.initialContent)

const handleSave = () => {
  emit('save', content.value)
  emit('close')
}

// Handle escape key
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 z-50 bg-slate-900"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-slate-700">
      <h2 class="text-xl font-semibold text-white">Edit Ticket Notes</h2>
      <div class="flex items-center gap-4">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm font-medium text-slate-300 hover:text-white transition-colors"
        >
          Cancel
        </button>
        <button
          @click="handleSave"
          class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Save Changes
        </button>
      </div>
    </div>

    <!-- Editor Container -->
    <div class="h-[calc(100vh-73px)] p-6">
      <div class="h-full bg-slate-800 rounded-lg p-6">
        <textarea
          v-model="content"
          class="w-full h-full bg-transparent text-white resize-none focus:outline-none"
          placeholder="Enter your notes here..."
        ></textarea>
      </div>
    </div>
  </div>
</template> 