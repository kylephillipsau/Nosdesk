import { ref, computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import ticketService from '@/services/ticketService';
import { useRecentTicketsStore } from '@/stores/recentTickets';
import documentationService from '@/services/documentationService';

export interface TitleableDocument {
  id: string;
  title: string;
  icon: string;
  slug?: string;
}

export interface TitleableTicket {
  id: number;
  title: string;
}

export function useTitleManager() {
  const route = useRoute();
  
  // State
  const currentTicket = ref<TitleableTicket | null>(null);
  const currentDocument = ref<TitleableDocument | null>(null);
  const documentationTitle = ref<string | null>(null);
  const customTitle = ref<string | null>(null);
  const isTransitioning = ref(false);
  
  // Computed properties
  const isTicketView = computed(() => currentTicket.value !== null);
  const isDocumentView = computed(() => currentDocument.value !== null);
  
  const pageTitle = computed(() => {
    // Custom title takes precedence if set
    if (customTitle.value) {
      return customTitle.value;
    }
    
    // Documentation title is next in priority
    if (isDocumentView.value && documentationTitle.value) {
      return documentationTitle.value;
    }
    
    // Ticket title with ID
    if (isTicketView.value && currentTicket.value) {
      return `#${currentTicket.value.id} ${currentTicket.value.title}`;
    }
    
    // Get title from route meta
    const routeTitle = route.meta?.title as string;
    
    // Update document title
    const finalTitle = routeTitle || 'Nosdesk';
    document.title = `${finalTitle} | Nosdesk`;
    
    return finalTitle;
  });
  
  // Watch for route changes
  watch(
    () => route.name,
    () => {
      // Reset custom title when route changes
      customTitle.value = null;
    }
  );
  
  // Methods
  const setCustomTitle = (title: string | null) => {
    customTitle.value = title;
    if (title) {
      document.title = `${title} | Nosdesk`;
    }
  };
  
  const setTicket = (ticketData: TitleableTicket | null) => {
    currentTicket.value = ticketData;
    if (ticketData) {
      document.title = `#${ticketData.id} ${ticketData.title} | Nosdesk`;
    }
  };
  
  const setDocument = (documentData: TitleableDocument | null) => {
    currentDocument.value = documentData;
    if (documentData) {
      documentationTitle.value = documentData.title;
      setCustomTitle(documentData.title);
    }
  };
  
  // Preview the ticket title as the user types (real-time updates)
  const previewTicketTitle = (newTitle: string) => {
    if (currentTicket.value) {
      console.log(`useTitleManager: Previewing ticket #${currentTicket.value.id} title as "${newTitle}"`);
      
      // Update the title in the current ticket for UI display
      currentTicket.value.title = newTitle;
      
      // Update the title in the recent tickets store immediately for real-time preview
      const recentTicketsStore = useRecentTicketsStore();
      recentTicketsStore.updateTicketData(currentTicket.value.id, {
        title: newTitle
      });
      
      // Update the document title for real-time feedback
      document.title = `#${currentTicket.value.id} ${newTitle} | Nosdesk`;
    }
  };
  
  // Preview the document title as the user types (real-time updates)
  const previewDocumentTitle = (newTitle: string) => {
    if (currentDocument.value) {
      console.log(`useTitleManager: Previewing document title as "${newTitle}"`);
      
      // Update the title in the current document for UI display
      currentDocument.value.title = newTitle;
      documentationTitle.value = newTitle;
      
      // Update the document title for real-time feedback
      document.title = `${newTitle} | Nosdesk`;
    }
  };
  
  const updateTicketTitle = async (newTitle: string) => {
    if (currentTicket.value) {
      console.log(`useTitleManager: Updating ticket #${currentTicket.value.id} title to "${newTitle}"`);
      
      // Update the title in the current ticket
      currentTicket.value.title = newTitle;
      
      // Update the title in the recent tickets store immediately
      const recentTicketsStore = useRecentTicketsStore();
      recentTicketsStore.updateTicketData(currentTicket.value.id, {
        title: newTitle
      });
      
      // Update the document title
      document.title = `#${currentTicket.value.id} ${newTitle} | Nosdesk`;
      
      // Save the title change to the backend
      try {
        await ticketService.updateTicket(currentTicket.value.id, { title: newTitle });
        console.log(`useTitleManager: Successfully updated ticket title in backend`);
      } catch (error) {
        console.error(`Error updating ticket title:`, error);
        // If the backend update fails, we don't revert the UI changes
        // because the user might still be editing the title
      }
    }
  };
  
  const updateDocumentTitle = async (newTitle: string) => {
    if (currentDocument.value) {
      console.log(`useTitleManager: Updating document title to "${newTitle}"`);
      
      // Update the title in the current document for UI display
      currentDocument.value.title = newTitle;
      documentationTitle.value = newTitle;
      setCustomTitle(newTitle);
      
      // Generate a slug from the title
      const slug = newTitle.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
      
      try {
        // First, fetch the current document to get all its properties
        const currentDocId = currentDocument.value.id;
        const existingDoc = await documentationService.getArticleById(currentDocId);
        
        if (!existingDoc) {
          console.error(`Could not find document with ID ${currentDocId}`);
          return;
        }
        
        // Save the updated document to the backend, preserving all existing properties
        const updatedDocument = await documentationService.saveArticle({
          ...existingDoc,
          title: newTitle,
          slug: slug,
          children: existingDoc.children || []
        });
        
        if (updatedDocument) {
          console.log(`useTitleManager: Successfully updated document title and slug in backend`);
          // Update the slug in the current document
          if (currentDocument.value) {
            currentDocument.value.slug = slug;
          }
        }
      } catch (error) {
        console.error('Error updating document title and slug:', error);
      }
    }
  };
  
  const updateDocumentIcon = async (newIcon: string) => {
    if (currentDocument.value) {
      console.log(`useTitleManager: Updating document icon to "${newIcon}"`);
      
      // Update the icon in the current document for UI display
      currentDocument.value.icon = newIcon;
      
      try {
        // First, fetch the current document to get all its properties
        const currentDocId = currentDocument.value.id;
        const existingDoc = await documentationService.getArticleById(currentDocId);
        
        if (!existingDoc) {
          console.error(`Could not find document with ID ${currentDocId}`);
          return;
        }
        
        // Save the updated document to the backend, preserving all existing properties
        const updatedDocument = await documentationService.saveArticle({
          ...existingDoc,
          icon: newIcon,
          children: existingDoc.children || []
        });
        
        if (updatedDocument) {
          console.log(`useTitleManager: Successfully updated document icon in backend`);
        }
      } catch (error) {
        console.error('Error updating document icon:', error);
      }
    }
  };
  
  const startTransition = () => {
    isTransitioning.value = true;
  };
  
  const endTransition = () => {
    isTransitioning.value = false;
  };
  
  const clearTicket = () => {
    currentTicket.value = null;
  };
  
  const clearDocument = () => {
    currentDocument.value = null;
    documentationTitle.value = null;
  };
  
  return {
    // State
    currentTicket,
    currentDocument,
    documentationTitle,
    isTransitioning,
    
    // Computed
    pageTitle,
    isTicketView,
    isDocumentView,
    
    // Methods
    setCustomTitle,
    setTicket,
    setDocument,
    previewTicketTitle,
    previewDocumentTitle,
    updateTicketTitle,
    updateDocumentTitle,
    updateDocumentIcon,
    startTransition,
    endTransition,
    clearTicket,
    clearDocument
  };
} 