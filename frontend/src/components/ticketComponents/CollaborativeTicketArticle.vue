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

// Use binary content for Yjs document
const content = ref('');
const router = useRouter();
const isLoading = ref(false);
const isBinaryUpdate = ref(true);

// Load initial content from backend
onMounted(async () => {
  isLoading.value = true;
  console.log('Attempting to load content for ticket', props.ticketId, 'from URL:', `${API_BASE_URL}/collaboration/article/ticket-${props.ticketId}`);
  try {
    const response = await axios.get(`${API_BASE_URL}/collaboration/article/ticket-${props.ticketId}`);
    console.log('Response received:', response);
    if (response.data.content) {
      content.value = response.data.content;
      console.log('Loaded initial content for ticket', props.ticketId, 'Content length:', content.value.length);
    } else {
      console.log('No initial content found for ticket', props.ticketId);
      content.value = '';
    }
  } catch (error: any) {
    console.error('Error loading initial content for ticket', props.ticketId, ':', error);
    if (error.response) {
      console.error('Response status:', error.response.status);
      console.error('Response data:', error.response.data);
    } else if (error.request) {
      console.error('No response received:', error.request);
    } else {
      console.error('Error setting up request:', error.message);
    }
    content.value = '';
  } finally {
    isLoading.value = false;
    emit('initialization-complete');
    console.log('Initialization complete for ticket', props.ticketId);
  }
});

// Handle expand to full page editor
const handleExpand = () => {
  router.push({ 
    path: '/documentation', 
    query: { ticketId: String(props.ticketId) } 
  });
};

// Save content to backend on update
const handleContentChange = async (newValue: string) => {
  content.value = newValue;
  console.log('Attempting to save content for ticket', props.ticketId, 'Content length:', newValue.length);
  try {
    const response = await axios.post(`${API_BASE_URL}/collaboration/sync`, {
      doc_id: `ticket-${props.ticketId}`,
      content: newValue
    });
    console.log('Content saved successfully for ticket', props.ticketId, 'Response:', response.data);
  } catch (error: any) {
    console.error('Error saving content for ticket', props.ticketId, ':', error);
    if (error.response) {
      console.error('Response status:', error.response.status);
      console.error('Response data:', error.response.data);
    } else if (error.request) {
      console.error('No response received:', error.request);
    } else {
      console.error('Error setting up request:', error.message);
    }
  }
};
</script>

<template>
  <div class="bg-slate-800 rounded-2xl p-2 pt-3 shadow-lg flex flex-col gap-2 w-full h-auto">
    <div class="text-lg font-medium text-slate-100 px-2 flex justify-between items-center">
      <h2 class="text-lg font-medium text-slate-100 px-2">Ticket Notes</h2>
      <button
        @click="handleExpand"
        class="p-1.5 py-1 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
        title="Open full editor"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h4a1 1 0 010 2H6.414l2.293 2.293a1 1 0 11-1.414 1.414L5 6.414V8a1 1 0 01-2 0V4zm9 1a1 1 0 010-2h4a1 1 0 011 1v4a1 1 0 01-2 0V6.414l-2.293 2.293a1 1 0 11-1.414-1.414L13.586 5H12zm-9 7a1 1 0 012 0v1.586l2.293-2.293a1 1 0 111.414 1.414L6.414 15H8a1 1 0 010 2H4a1 1 0 01-1-1v-4zm13-1a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 010-2h1.586l-2.293-2.293a1 1 0 111.414-1.414L15 13.586V12a1 1 0 011-1z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
    <div v-if="isLoading" class="p-4 flex flex-grow justify-center items-center">
      <div class="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500"></div>
    </div>
    <div v-else class="flex-grow flex w-full">
      <CollaborativeEditor
        v-model="content"
        :doc-id="`ticket-${ticketId}`"
        :is-binary-update="true"
        @update:model-value="handleContentChange"
        class="flex-grow w-full"
      />
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  position: relative;
  height: auto;
  width: 100%;
  overflow: visible;
}
</style> 