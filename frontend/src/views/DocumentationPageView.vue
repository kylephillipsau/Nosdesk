<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import MarkdownEditor from "@/components/MarkdownEditor.vue";
import { usePageTitle } from "@/composables/usePageTitle";
import documentationService from "@/services/documentationService";
import type { Article, Category } from "@/services/documentationService";
import BackButton from '@/components/common/BackButton.vue';

const route = useRoute();
const router = useRouter();
const article = ref<Article | null>(null);
const category = ref<Category | null>(null);
const isLoading = ref(true);
const showSuccessMessage = ref(false);
const isSaving = ref(false);
const saveMessage = ref("Document saved successfully");
const { setCustomTitle } = usePageTitle();
const isTicketNote = ref(false);
const isCategory = ref(false);
const ticketId = ref<string | null>(null);
const categoryId = ref<string | null>(null);

// Content editing
const editContent = ref("");
const editTitle = ref("");

// Document icon
const documentIcon = ref('📄');

// Create a document object for the header
const document = computed(() => {
  if (isCategory.value && category.value) {
    return {
      id: category.value.id,
      title: category.value.name,
      icon: category.value.icon || '📁'
    };
  } else if (!article.value) {
    return null;
  }
  
  return {
    id: article.value.id,
    title: editTitle.value || article.value.title,
    icon: documentIcon.value
  };
});

// Define emits for article data
const emit = defineEmits<{
  (e: 'update:title', title: string): void;
  (e: 'update:document', document: { id: string; title: string; icon: string } | null): void;
}>();

// Fetch article data
const fetchArticle = async (id: string) => {
  isLoading.value = true;
  
  // Check if this is a category note
  if (route.name === 'documentation-category') {
    isCategory.value = true;
    categoryId.value = route.params.categoryId as string;
    
    try {
      // Fetch category data
      const fetchedCategory = await documentationService.getCategoryById(categoryId.value);
      
      if (fetchedCategory) {
        category.value = fetchedCategory;
        editContent.value = fetchedCategory.content || "";
        editTitle.value = fetchedCategory.name || "";
        documentIcon.value = fetchedCategory.icon || '📁';
        
        // Emit the title and document
        console.log('Emitting title from fetchArticle (category):', fetchedCategory.name);
        emit('update:title', fetchedCategory.name || "");
        emit('update:document', document.value);
        setCustomTitle(fetchedCategory.name || "");
      } else {
        console.error("Category not found");
        category.value = null;
      }
    } catch (error) {
      console.error("Error fetching category:", error);
      category.value = null;
    } finally {
      isLoading.value = false;
    }
    
    return;
  }
  
  // Check if this is a ticket note
  if (id.startsWith('ticket-')) {
    isTicketNote.value = true;
    ticketId.value = id.replace('ticket-', '');
    
    try {
      // Fetch ticket data instead of article data
      // TODO: Replace with actual API call
      const ticketData = (await import("@/data/tickets.json")).default;
      const ticket = ticketData.tickets.find((t: any) => t.id === Number(ticketId.value));
      
      if (ticket) {
        // Create an article-like object from the ticket
        article.value = {
          id: `ticket-${ticket.id}`,
          title: `Ticket #${ticket.id} Notes`,
          description: ticket.title || '',
          content: ticket.articleContent || '',
          category: 'tickets',
          author: ticket.assignee || 'System',
          lastUpdated: ticket.modified || new Date().toISOString(),
          status: 'published',
          icon: '🎫' // Default icon for ticket notes
        };
        
        editContent.value = article.value.content || "";
        editTitle.value = article.value.title || "";
        documentIcon.value = article.value.icon || '🎫';
        
        // Emit the title and document
        console.log('Emitting title from fetchArticle (ticket):', article.value.title);
        emit('update:title', article.value.title || "");
        emit('update:document', document.value);
        setCustomTitle(article.value.title || "");
      } else {
        console.error("Ticket not found");
        article.value = null;
      }
    } catch (error) {
      console.error("Error fetching ticket:", error);
      article.value = null;
    } finally {
      isLoading.value = false;
    }
  } else {
    // Regular documentation article
    try {
      const fetchedArticle = await documentationService.getArticleById(id);
      if (fetchedArticle) {
        article.value = fetchedArticle;
        editContent.value = fetchedArticle.content || "";
        editTitle.value = fetchedArticle.title || "";
        documentIcon.value = fetchedArticle.icon || '📄'; // Use article icon or default
        
        // Immediately emit the title and document to ensure it's displayed in the header
        console.log('Emitting title from fetchArticle:', fetchedArticle.title);
        emit('update:title', fetchedArticle.title || "");
        emit('update:document', document.value);
        setCustomTitle(fetchedArticle.title || "");
      } else {
        console.error("Article not found");
        article.value = null;
      }
    } catch (error) {
      console.error("Error fetching article:", error);
      article.value = null;
    } finally {
      isLoading.value = false;
    }
  }
};

// Watch article changes to emit updates
watch(article, (newArticle) => {
  if (newArticle) {
    console.log('Emitting title from article watch:', editTitle.value);
    emit('update:title', editTitle.value);
    setCustomTitle(editTitle.value);
  } else {
    emit('update:title', '');
  }
}, { immediate: true });

// Watch document changes to emit updates
watch(document, (newDocument) => {
  console.log('Emitting document from document watch:', newDocument);
  emit('update:document', newDocument);
}, { immediate: true });

// Handle content update from the markdown editor
const updateContent = (newContent: string) => {
  editContent.value = newContent;
  // Auto-save after a short delay
  saveArticleDebounced();
};

// Handle title update
const updateTitle = (newTitle: string) => {
  editTitle.value = newTitle;
  
  // Update the title in the header
  if (article.value) {
    console.log('Emitting title from updateTitle:', newTitle);
    emit('update:title', newTitle);
    setCustomTitle(newTitle);
  }
  
  // Auto-save after a short delay
  saveArticleDebounced();
};

// Handle document title update from header
const updateDocumentTitle = (newTitle: string) => {
  editTitle.value = newTitle;
  
  // Auto-save after a short delay
  saveArticleDebounced();
};

// Handle document icon update
const updateDocumentIcon = (newIcon: string) => {
  documentIcon.value = newIcon;
  
  // Update the document object and emit changes
  if (article.value) {
    emit('update:document', document.value);
    
    // Auto-save after a short delay
    saveArticleDebounced();
  }
};

// Debounced save function
let saveTimeout: number | null = null;
const saveArticleDebounced = () => {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
  }

  saveTimeout = setTimeout(() => {
    saveMessage.value = "Document saved successfully";
    saveArticle();
  }, 1000) as unknown as number;
};

// Save article changes
const saveArticle = async () => {
  if (!article.value) return;

  try {
    isSaving.value = true;
    
    // Update the article with new content, title, and icon
    const updatedArticle: Article = {
      ...article.value,
      title: editTitle.value,
      content: editContent.value,
      lastUpdated: new Date().toISOString(),
      icon: documentIcon.value
    };

    if (isTicketNote.value && ticketId.value) {
      // Save ticket note
      // TODO: Replace with actual API call to save ticket note
      console.log(`Saving ticket note for ticket #${ticketId.value}`);
      
      // For now, just update the local article
      article.value = updatedArticle;
      
      // In a real implementation, you would call a ticket service method here
      // Example: await ticketService.saveTicketNotes(ticketId.value, editContent.value);
    } else {
      // Save regular documentation article
      const savedArticle = await documentationService.saveArticle(updatedArticle);
      article.value = savedArticle;
    }

    // Show success message
    showSuccessMessage.value = true;
    setTimeout(() => {
      showSuccessMessage.value = false;
    }, 3000);
  } catch (error) {
    console.error("Error saving article:", error);
    saveMessage.value = "Error saving document";
    showSuccessMessage.value = true;
    setTimeout(() => {
      showSuccessMessage.value = false;
    }, 3000);
  } finally {
    isSaving.value = false;
  }
};

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

// Add a computed property for the fallback route
const fallbackRoute = computed(() => {
  if (isTicketNote.value && ticketId.value) {
    return `/tickets/${ticketId.value}`;
  } else {
    return '/documentation';
  }
});

// Add a computed property for the back button label
const backButtonLabel = computed(() => {
  return isTicketNote.value ? 'Back to Ticket' : 'Back to Documentation';
});

// Watch for changes in the article title
watch(() => article.value?.title, (newTitle) => {
  if (newTitle && newTitle !== editTitle.value) {
    editTitle.value = newTitle;
  }
});

onMounted(() => {
  // Check if we're on a category page
  if (route.name === 'documentation-category' && route.params.categoryId) {
    fetchArticle('');  // The ID doesn't matter for categories, it's handled in the function
  } else if (route.params.id) {
    fetchArticle(route.params.id as string);
  }
  
  // Ensure title and document are emitted on mount if we already have them
  if (article.value && article.value.title) {
    console.log('Emitting title on mount:', article.value.title);
    emit('update:title', article.value.title);
    setCustomTitle(article.value.title);
    
    // Initialize document icon if available
    if (article.value.icon) {
      documentIcon.value = article.value.icon;
    }
    
    // Also emit the document object
    if (document.value) {
      console.log('Emitting document on mount:', document.value);
      emit('update:document', document.value);
    }
  }
});

// Save the document
const saveDocument = async () => {
  if (!editContent.value) {
    return;
  }
  
  isSaving.value = true;
  showSuccessMessage.value = false;
  
  try {
    if (isCategory.value && category.value) {
      // Save category content
      const updatedCategory = await documentationService.saveCategoryContent(
        category.value.id,
        editContent.value,
        category.value.author || 'System Admin'
      );
      
      if (updatedCategory) {
        category.value = updatedCategory;
        saveMessage.value = "Category documentation saved successfully";
        showSuccessMessage.value = true;
      }
    } else if (isTicketNote.value && ticketId.value) {
      // Save ticket note
      // TODO: Replace with actual API call to save ticket note
      console.log(`Saving ticket note for ticket #${ticketId.value}`);
      
      // For now, just update the local article
      if (article.value) {
        article.value.content = editContent.value;
        article.value.lastUpdated = new Date().toISOString();
        saveMessage.value = "Ticket notes saved successfully";
        showSuccessMessage.value = true;
      }
    } else if (article.value) {
      // Save regular documentation article
      const updatedArticle: Article = {
        ...article.value,
        title: editTitle.value,
        content: editContent.value,
        lastUpdated: new Date().toISOString(),
        icon: documentIcon.value
      };
      
      const savedArticle = await documentationService.saveArticle(updatedArticle);
      article.value = savedArticle;
      saveMessage.value = "Document saved successfully";
      showSuccessMessage.value = true;
    }
  } catch (error) {
    console.error("Error saving document:", error);
    saveMessage.value = "Error saving document";
    showSuccessMessage.value = true;
  } finally {
    isSaving.value = false;
    
    // Show success message briefly
    setTimeout(() => {
      showSuccessMessage.value = false;
    }, 3000);
  }
};
</script>

<template>
  <div class="bg-slate-900 flex flex-col">
    <!-- Main content area with a single scrollbar -->
    <main class="flex-1 overflow-auto">
      <div class="flex items-center justify-between px-6 py-4 border-b border-slate-700">
        <BackButton :fallbackRoute="fallbackRoute" :label="backButtonLabel" />
        
        <div v-if="article" class="text-xs text-slate-400 flex items-center gap-4">
          <span>{{ article.author }}</span>
          <span>Updated {{ formatDate(article.lastUpdated) }}</span>
          <span v-if="isSaving" class="text-blue-400 flex items-center gap-1">
            <svg class="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Saving...
          </span>
        </div>
      </div>

      <div v-if="article" class="flex justify-center h-full p-4">
        <MarkdownEditor
          v-model="editContent"
          @update:modelValue="updateContent"
          @save="saveArticle"
          class="h-full max-w-7xl mx-auto"
        />
      </div>

      <div
        v-else-if="isLoading"
        class="flex justify-center items-center h-full"
      >
        <div
          class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"
        ></div>
      </div>

      <div v-else class="p-6 text-center text-slate-400">Article not found</div>
    </main>

    <!-- Success message toast -->
    <div
      v-if="showSuccessMessage"
      class="fixed bottom-4 right-4 bg-green-600 text-white px-4 py-2 rounded-md shadow-lg"
    >
      {{ saveMessage }}
    </div>
  </div>
</template>

<style scoped>
/* No additional styles needed */
</style>