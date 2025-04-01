<!-- DocumentationArticleEdit.vue -->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import CollaborativeEditor from '@/components/CollaborativeEditor.vue';
import StatusBadge from '@/components/StatusBadge.vue';

interface Article {
  id: string;
  title: string;
  content: string;
  author: string;
  lastUpdated: string;
  status: 'published' | 'draft';
  category: string;
}

const route = useRoute();
const router = useRouter();

const article = ref<Article | null>(null);
const isLoading = ref(true);
const error = ref<string | null>(null);
const isSaving = ref(false);
const showSuccessMessage = ref(false);
const isBinaryUpdate = ref(false);

// Form fields
const title = ref('');
const content = ref('');
const category = ref('');
const status = ref<'published' | 'draft'>('draft');

// Create a computed doc ID for collaborative editing
const docId = computed(() => {
  if (article.value) {
    return `documentation-${article.value.id}`;
  }
  return 'documentation-new';
});

// Load article data
onMounted(async () => {
  try {
    // Check if we're editing an existing article
    if (route.params.id && route.params.id !== 'new') {
      // First try to get a binary update from the collaborative system
      try {
        const response = await fetch(`${import.meta.env.VITE_API_URL}/collaboration/state/${docId.value}`, {
          method: 'GET',
          headers: { Accept: 'application/octet-stream' }
        });
        
        if (response.ok) {
          const data = await response.arrayBuffer();
          if (data.byteLength > 0) {
            console.log('Loaded binary update for document:', docId.value);
            
            // Convert binary to base64 for the editor
            const bytes = new Uint8Array(data);
            const binary = bytes.reduce((acc, byte) => acc + String.fromCharCode(byte), '');
            const base64 = window.btoa(binary);
            
            content.value = base64;
            isBinaryUpdate.value = true;
          }
        }
      } catch (err) {
        console.error('Error fetching binary update:', err);
      }
      
      // Mock API call - replace with actual API call
      article.value = {
        id: route.params.id as string,
        title: 'Introduction to NosDesk',
        content: `
# Introduction to NosDesk

Welcome to NosDesk, your modern helpdesk solution. This guide will help you understand the core features and get started with using NosDesk effectively.

## What is NosDesk?

NosDesk is a comprehensive helpdesk solution designed to streamline your support operations. It provides:

- **Ticket Management**: Efficiently handle support requests
- **Knowledge Base**: Create and maintain documentation
- **Asset Management**: Track devices and software
- **Team Collaboration**: Work together seamlessly

## Key Features

### 1. Smart Ticket Routing

NosDesk automatically routes tickets to the right team members based on:

\`\`\`typescript
interface RoutingRule {
  priority: number;
  department: string;
  skills: string[];
  availability: boolean;
}
\`\`\`

### 2. Real-time Analytics

Monitor your support performance with real-time metrics:

- Response time
- Resolution rate
- Customer satisfaction
- Team workload

## Getting Started

1. Create your account
2. Set up your team
3. Configure workflows
4. Start managing tickets

> Pro tip: Take time to set up your automation rules early. They'll save you hours of work later.

For more detailed instructions, check out our [Quick Start Guide](/documentation/quick-start).
        `,
        author: 'NosDesk Team',
        lastUpdated: '2024-03-20',
        status: 'published',
        category: 'getting-started'
      };

      // Only populate form fields from article if we don't have a binary update
      if (!isBinaryUpdate.value) {
        content.value = article.value.content;
      }
      
      // Always set these fields
      title.value = article.value.title;
      category.value = article.value.category;
      status.value = article.value.status;
    } else {
      // Creating a new article
      article.value = {
        id: 'new',
        title: '',
        content: '# New Article\n\nStart writing your documentation here...',
        author: 'NosDesk Team', // This would come from the current user
        lastUpdated: new Date().toISOString(),
        status: 'draft',
        category: ''
      };

      // Populate form fields
      title.value = article.value.title;
      content.value = article.value.content;
      category.value = article.value.category;
      status.value = article.value.status;
    }
  } catch (e) {
    error.value = 'Failed to load article';
  } finally {
    isLoading.value = false;
  }
});

// Save the article
const saveArticle = async () => {
  isSaving.value = true;
  
  try {
    // Update article with form values
    if (article.value) {
      article.value.title = title.value;
      article.value.content = content.value;
      article.value.category = category.value;
      article.value.status = status.value;
      article.value.lastUpdated = new Date().toISOString();
    }
    
    // Mock API call - replace with actual API call
    // await api.saveArticle(article.value);
    
    // Show success message
    showSuccessMessage.value = true;
    setTimeout(() => {
      showSuccessMessage.value = false;
    }, 3000);
    
    // If this is a new article, redirect to the edit page with the new ID
    if (route.params.id === 'new') {
      // In a real app, you'd get the new ID from the API response
      const newId = 'new-article-' + Date.now();
      router.replace(`/documentation/edit/${newId}`);
    }
  } catch (e) {
    error.value = 'Failed to save article';
  } finally {
    isSaving.value = false;
  }
};

// Handle content update from the editor
const updateContent = (newContent: string) => {
  content.value = newContent;
};

// Toggle article status
const toggleStatus = () => {
  status.value = status.value === 'published' ? 'draft' : 'published';
};

// Categories for the dropdown
const categories = [
  'getting-started',
  'user-guide',
  'administration',
  'troubleshooting',
  'api-reference',
  'release-notes'
];
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div v-if="isLoading" class="flex items-center justify-center h-full">
      <div class="text-slate-400">Loading...</div>
    </div>

    <div v-else-if="error" class="flex items-center justify-center h-full">
      <div class="text-red-400">{{ error }}</div>
    </div>

    <div v-else-if="article" class="max-w-4xl mx-auto px-6 py-8">
      <!-- Header with title input and status -->
      <div class="flex items-start justify-between mb-8">
        <div class="w-full">
          <input
            v-model="title"
            type="text"
            placeholder="Article Title"
            class="w-full text-3xl font-medium text-white bg-transparent border-b border-slate-700 pb-2 focus:outline-none focus:border-blue-500"
          />
          
          <div class="flex items-center gap-4 mt-4">
            <!-- Category dropdown -->
            <div class="relative">
              <select
                v-model="category"
                class="appearance-none bg-slate-800 text-slate-300 py-1 px-3 pr-8 rounded border border-slate-700 focus:outline-none focus:border-blue-500"
              >
                <option value="" disabled>Select Category</option>
                <option v-for="cat in categories" :key="cat" :value="cat">
                  {{ cat.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase()) }}
                </option>
              </select>
              <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-slate-400">
                <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                  <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
                </svg>
              </div>
            </div>
            
            <!-- Status toggle -->
            <button
              @click="toggleStatus"
              class="flex items-center gap-2 text-sm py-1 px-3 rounded border border-slate-700 hover:bg-slate-700 transition-colors"
            >
              <span>Status:</span>
              <StatusBadge 
                type="status" 
                :value="status === 'published' ? 'open' : 'in-progress'"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- Success message -->
      <div
        v-if="showSuccessMessage"
        class="fixed top-4 right-4 bg-green-900 text-green-100 px-4 py-2 rounded shadow-lg z-50 transition-opacity"
      >
        Article saved successfully!
      </div>

      <!-- Collaborative Editor -->
      <CollaborativeEditor
        v-model="content"
        :doc-id="docId"
        :is-binary-update="isBinaryUpdate"
        placeholder="Start writing your documentation here..."
        @update:modelValue="updateContent"
      />
      
      <!-- Action buttons -->
      <div class="flex justify-end mt-6 gap-4">
        <button
          @click="router.push('/documentation')"
          class="px-4 py-2 text-slate-300 hover:text-white transition-colors"
        >
          Cancel
        </button>
        <button
          @click="saveArticle"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded shadow transition-colors flex items-center gap-2"
          :disabled="isSaving"
        >
          <span v-if="isSaving">Saving...</span>
          <span v-else>Save Article</span>
        </button>
      </div>
    </div>

    <div v-else class="flex items-center justify-center h-full">
      <div class="text-slate-400">Article not found</div>
    </div>
  </div>
</template> 