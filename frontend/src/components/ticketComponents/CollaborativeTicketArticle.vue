<!-- CollaborativeTicketArticle.vue -->
<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import CollaborativeEditor from '@/components/CollaborativeEditor.vue';
import { API_BASE_URL } from '@/config';
import axios from 'axios';

// Define props
interface Props {
  initialContent?: string;
  ticketId: number;
  initializing?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  initialContent: '',
  initializing: false,
});

const emit = defineEmits<{
  'update:content': [content: string];
  'initialization-complete': [];
}>();

// Always use an empty document for now - ignore initialContent
const content = ref('{"type":"doc","content":[{"type":"paragraph"}]}');
const router = useRouter();
const isLoading = ref(false);
const isBinaryUpdate = ref(false);

// SIMPLIFIED: Just emit initialization complete
onMounted(() => {
  // Always emit initialization complete
  emit('initialization-complete');
});

// Handle expand to full page editor
const handleExpand = () => {
  router.push({ 
    path: '/documentation', 
    query: { ticketId: String(props.ticketId) } 
  });
};

// SIMPLIFIED: Just update local content, don't sync to server or parent
const handleContentChange = (newValue: string) => {
  content.value = newValue;
  // Don't emit updates to parent
};

// Disable watching initialContent changes
/* 
watch(() => props.initialContent, (newValue) => {
  // Only update if different to avoid loops
  if (newValue !== content.value) {
    content.value = newValue || '';
  }
}, { immediate: true });
*/
</script>

<template>
  <div class="bg-slate-800 rounded-2xl p-2 shadow-lg">
    <div class="text-lg font-medium text-slate-100 p-4 py-2 flex justify-between items-center">
      <div class="flex items-center gap-4">
        <span>Ticket Notes</span>
      </div>
      <button
        @click="handleExpand"
        class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
        title="Open full editor"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h4a1 1 0 010 2H6.414l2.293 2.293a1 1 0 11-1.414 1.414L5 6.414V8a1 1 0 01-2 0V4zm9 1a1 1 0 010-2h4a1 1 0 011 1v4a1 1 0 01-2 0V6.414l-2.293 2.293a1 1 0 11-1.414-1.414L13.586 5H12zm-9 7a1 1 0 012 0v1.586l2.293-2.293a1 1 0 111.414 1.414L6.414 15H8a1 1 0 010 2H4a1 1 0 01-1-1v-4zm13-1a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 010-2h1.586l-2.293-2.293a1 1 0 111.414-1.414L15 13.586V12a1 1 0 011-1z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
    <div v-if="isLoading" class="p-4 flex justify-center items-center">
      <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
    </div>
    <div v-else>
      <CollaborativeEditor
        v-model="content"
        :doc-id="`ticket-${ticketId}`"
        :is-binary-update="false"
        placeholder="Enter ticket notes here..."
        @update:model-value="handleContentChange"
      />
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  position: relative;
}
</style> 