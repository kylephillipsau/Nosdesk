import { ref, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';

// Event types that match the backend
export interface TicketEvent {
  type: 'TicketUpdated' | 'CommentAdded' | 'CommentDeleted' | 'AttachmentAdded' | 'AttachmentDeleted' | 
        'DeviceLinked' | 'DeviceUnlinked' | 'ProjectAssigned' | 'ProjectUnassigned' | 
        'TicketLinked' | 'TicketUnlinked' | 'Heartbeat';
  data: any;
}

export interface TicketUpdatedEvent {
  ticket_id: number;
  field: string;
  value: any;
  updated_by: string;
  timestamp: string;
}

export interface CommentAddedEvent {
  ticket_id: number;
  comment: any;
  timestamp: string;
}

export interface CommentDeletedEvent {
  ticket_id: number;
  comment_id: number;
  timestamp: string;
}

// Global SSE state - simple and reactive
const eventSource = ref<EventSource | null>(null);
const isConnected = ref(false);
const isConnecting = ref(false);
const lastError = ref<string | null>(null);
const eventListeners = new Map<string, Set<(data: any) => void>>();

// Simple composable for SSE
export function useSSE() {
  const authStore = useAuthStore();

  // Get SSE token from backend
  const getSseToken = async (): Promise<string> => {
    const response = await fetch('/api/events/token', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${authStore.token}`,
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      throw new Error(`Failed to get SSE token: ${response.status}`);
    }

    const data = await response.json();
    return data.sse_token;
  };

  // Connect to SSE
  const connect = async (ticketId?: number) => {
    // Don't connect if already connected or connecting
    if (eventSource.value || isConnecting.value) {
      return;
    }

    if (!authStore.token) {
      lastError.value = 'No authentication token';
      return;
    }

    isConnecting.value = true;
    lastError.value = null;

    try {
      // Get SSE token
      const sseToken = await getSseToken();

      // Build URL
      const params = new URLSearchParams({ sse_token: sseToken });
      if (ticketId) {
        params.append('ticket_id', ticketId.toString());
      }
      const url = `/api/events/tickets?${params.toString()}`;

      // Create EventSource
      eventSource.value = new EventSource(url);

      // Handle connection events
      eventSource.value.onopen = () => {
        console.log('SSE: Connected');
        isConnected.value = true;
        isConnecting.value = false;
        lastError.value = null;
      };

      eventSource.value.onerror = (error) => {
        console.error('SSE: Connection error', error);
        isConnected.value = false;
        isConnecting.value = false;
        lastError.value = 'Connection failed';
        
        // Clean up
        if (eventSource.value) {
          eventSource.value.close();
          eventSource.value = null;
        }
      };

      // Handle different event types
      eventSource.value.addEventListener('ticket-updated', (event) => {
        try {
          const data = JSON.parse(event.data);
          console.log('SSE: Parsed ticket-updated event data:', data);
          emit('ticket-updated', data);
        } catch (e) {
          console.error('SSE: Failed to parse ticket-updated event', e);
          console.log('SSE: Raw event data:', event.data);
        }
      });

      eventSource.value.addEventListener('comment-added', (event) => {
        try {
          const data = JSON.parse(event.data);
          emit('comment-added', data);
        } catch (e) {
          console.error('SSE: Failed to parse comment-added event', e);
        }
      });

      eventSource.value.addEventListener('comment-deleted', (event) => {
        try {
          const data = JSON.parse(event.data);
          emit('comment-deleted', data);
        } catch (e) {
          console.error('SSE: Failed to parse comment-deleted event', e);
        }
      });

      eventSource.value.addEventListener('device-linked', (event) => {
        try {
          const data = JSON.parse(event.data);
          emit('device-linked', data);
        } catch (e) {
          console.error('SSE: Failed to parse device-linked event', e);
        }
      });

      eventSource.value.addEventListener('device-unlinked', (event) => {
        try {
          const data = JSON.parse(event.data);
          emit('device-unlinked', data);
        } catch (e) {
          console.error('SSE: Failed to parse device-unlinked event', e);
        }
      });

    } catch (error) {
      console.error('SSE: Failed to connect', error);
      isConnecting.value = false;
      lastError.value = error instanceof Error ? error.message : 'Failed to connect';
    }
  };

  // Disconnect from SSE
  const disconnect = () => {
    if (eventSource.value) {
      eventSource.value.close();
      eventSource.value = null;
    }
    isConnected.value = false;
    isConnecting.value = false;
    lastError.value = null;
  };

  // Emit event to listeners
  const emit = (eventType: string, data: any) => {
    const listeners = eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(data);
        } catch (error) {
          console.error(`SSE: Error in ${eventType} listener`, error);
        }
      });
    }
  };

  // Add event listener
  const addEventListener = (eventType: string, listener: (data: any) => void) => {
    if (!eventListeners.has(eventType)) {
      eventListeners.set(eventType, new Set());
    }
    eventListeners.get(eventType)!.add(listener);
  };

  // Remove event listener
  const removeEventListener = (eventType: string, listener: (data: any) => void) => {
    const listeners = eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(listener);
      if (listeners.size === 0) {
        eventListeners.delete(eventType);
      }
    }
  };

  return {
    // State
    isConnected: computed(() => isConnected.value),
    isConnecting: computed(() => isConnecting.value),
    error: computed(() => lastError.value),
    
    // Methods
    connect,
    disconnect,
    addEventListener,
    removeEventListener
  };
} 