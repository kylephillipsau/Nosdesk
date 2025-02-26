<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import MarkdownEditor from '@/components/MarkdownEditor.vue';

// Define props
interface Props {
  initialContent?: string;
}

const props = withDefaults(defineProps<Props>(), {
  initialContent: '',
});

const emit = defineEmits<{
  'update:content': [content: string];
}>();

const content = ref(props.initialContent || '');
const router = useRouter();
const route = useRoute();

// Handle expand to full page editor
const handleExpand = () => {
  // Redirect to documentation page view with a special ticket ID format
  router.push(`/documentation/ticket-${route.params.id}`);
};

// Handle content update
const updateContent = (newContent: string) => {
  content.value = newContent;
  emit('update:content', newContent);
};

// Handle save
const handleSave = () => {
  emit('update:content', content.value);
};

// Watch for changes in initialContent
watch(() => props.initialContent, (newValue) => {
  if (newValue !== undefined && newValue !== content.value) {
    content.value = newValue;
  }
}, { immediate: true });

onMounted(() => {
  if (props.initialContent) {
    content.value = props.initialContent;
  }
});
</script>

<template>
  <div class="bg-slate-800 rounded-2xl p-2 shadow-lg">
    <div class="text-lg font-medium text-slate-100 p-4 py-2 flex justify-between items-center">
      <span>Ticket Notes</span>
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
    <div class="editor-wrapper">
      <MarkdownEditor
        v-model="content"
        @update:modelValue="updateContent"
        @save="handleSave"
        placeholder="Enter ticket notes here..."
      />
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  position: relative;
}

:deep(.markdown-editor) {
  border-radius: 0.5rem;
  min-height: 400px;
}

:deep(.ProseMirror h1) {
  margin-top: 0.5rem;
}
</style>