<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { EditorView } from 'prosemirror-view';
import { EditorState } from 'prosemirror-state';
import { schema } from '@/components/editor/schema';
import { ySyncPlugin, yCursorPlugin, yUndoPlugin, undo, redo } from 'y-prosemirror';
import { keymap } from 'prosemirror-keymap';
import { baseKeymap } from 'prosemirror-commands';
import 'prosemirror-view/style/prosemirror.css';

// Refs
const editorElement = ref<HTMLElement | null>(null);
const connectBtn = ref<HTMLElement | null>(null);
const statusRef = ref<HTMLElement | null>(null);
const isConnected = ref(false);
const docId = ref('test-document-1');

// Stores global references - directly mirrors the demo approach
let ydoc: Y.Doc | null = null;
let provider: WebsocketProvider | null = null;
let yXmlFragment: Y.XmlFragment | null = null;
let editorView: EditorView | null = null;

// Initialize editor - following the demo pattern exactly
const initEditor = () => {
  if (!editorElement.value) return;

  try {
    console.log('Initializing test editor with custom styling');
    
    // 1. Create Yjs document - not using refs for direct compatibility
    ydoc = new Y.Doc();
    
    // 2. Set up provider - using the exact format from the demo
    provider = new WebsocketProvider(
      'wss://demos.yjs.dev/ws',
      docId.value,
      ydoc
    );
    
    isConnected.value = true;
    
    // Set up log status (not in demo, but useful)
    provider.on('status', (event: { status: string }) => {
      if (statusRef.value) {
        statusRef.value.textContent = `Status: ${event.status}`;
      }
      console.log('Connection status:', event.status);
    });
    
    // 3. Get the XmlFragment - same as demo
    yXmlFragment = ydoc.getXmlFragment('prosemirror');
    
    // 4. Set up the editor - but with custom plugins instead of exampleSetup
    editorView = new EditorView(editorElement.value, {
      state: EditorState.create({
        schema,
        plugins: [
          ySyncPlugin(yXmlFragment),
          yCursorPlugin(provider.awareness),
          yUndoPlugin(),
          keymap({
            'Mod-z': undo,
            'Mod-y': redo,
            'Mod-Shift-z': redo
          }),
          keymap(baseKeymap) // Add basic keymap for editing functions
        ]
      })
    });
    
    // 5. Save instance for debugging - same as demo
    window.example = { provider, ydoc, yXmlFragment, editorView };
    
    console.log('Test editor initialized successfully');
  } catch (error) {
    console.error('Error initializing editor:', error);
  }
};

// Helper function to get random color for user cursors
const getRandomColor = () => {
  const colors = ['#f87171', '#fb923c', '#fbbf24', '#a3e635', '#34d399', '#22d3ee', '#60a5fa', '#a78bfa'];
  return colors[Math.floor(Math.random() * colors.length)];
};

// Toggle connection - using the exact same pattern as the demo
const toggleConnection = () => {
  if (!provider) return;
  
  if (provider.shouldConnect) {
    provider.disconnect();
    isConnected.value = false;
    if (connectBtn.value) {
      connectBtn.value.textContent = 'Connect';
    }
  } else {
    provider.connect();
    isConnected.value = true;
    if (connectBtn.value) {
      connectBtn.value.textContent = 'Disconnect';
    }
  }
};

// Change document ID for testing
const changeDocId = () => {
  const newId = prompt('Enter a new document ID:', docId.value);
  if (newId && newId !== docId.value) {
    // Clean up old connection
    cleanup();
    
    // Set new document ID and reinitialize
    docId.value = newId;
    // Need to wait for the next tick to ensure cleanup is complete
    setTimeout(() => {
      initEditor();
    }, 100);
  }
};

// Cleanup function to properly dispose resources
const cleanup = () => {
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  if (provider) {
    provider.disconnect();
    provider = null;
  }
  if (ydoc) {
    ydoc.destroy();
    ydoc = null;
  }
  yXmlFragment = null;
  
  // Clean up global references
  window.example = undefined;
};

// Initialize on mount
onMounted(() => {
  initEditor();
});

// Cleanup on unmount
onBeforeUnmount(() => {
  cleanup();
});

// Add to window for typescript
declare global {
  interface Window {
    example?: any;
  }
}
</script>

<template>
  <div class="test-editor">
    <div class="mb-4 flex justify-between items-center">
      <h2 class="text-xl font-bold">Test Editor ({{ docId }})</h2>
      <button 
        @click="changeDocId" 
        class="px-3 py-1 bg-blue-600 text-white rounded-md text-sm"
      >
        Change Document ID
      </button>
    </div>
    
    <p class="mb-4">This is a minimal test implementation of the yjs prosemirror collaborative editor configuration.</p>
    
    <div class="toolbar mb-4 flex items-center gap-4">
      <button 
        ref="connectBtn"
        id="y-connect-btn"
        @click="toggleConnection" 
        class="px-4 py-2 bg-slate-700 text-white rounded-md"
        :class="{ 'bg-green-700': isConnected, 'bg-red-700': !isConnected }"
      >
        {{ isConnected ? 'Disconnect' : 'Connect' }}
      </button>
      <span ref="statusRef" class="text-sm">Status: Loading...</span>
    </div>
    
    <div id="editor" ref="editorElement" class="border border-slate-600 min-h-[300px] rounded-md p-4 bg-slate-800"></div>
    
    <div class="mt-4 text-sm text-slate-400">
      <p>Note: The basic keymap is included but no fancy menus.</p>
    </div>
  </div>
</template>

<style scoped>
.test-editor {
  padding: 1rem;
  background-color: #1e293b;
  border-radius: 0.5rem;
  color: #e2e8f0;
}

/* ProseMirror styles */
:deep(.ProseMirror) {
  outline: none;
  min-height: 200px;
  color: #e2e8f0;
}

:deep(.ProseMirror p) {
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

:deep(.ProseMirror h1) {
  font-size: 1.5rem;
  font-weight: bold;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

:deep(.ProseMirror h2) {
  font-size: 1.25rem;
  font-weight: bold;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

:deep(.ProseMirror h3) {
  font-size: 1.125rem;
  font-weight: bold;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

:deep(.ProseMirror blockquote) {
  border-left: 3px solid #4b5563;
  padding-left: 1rem;
  margin-left: 0;
  margin-right: 0;
  color: #94a3b8;
}

:deep(.ProseMirror ul) {
  padding-left: 1.5rem;
  list-style-type: disc;
}

:deep(.ProseMirror ol) {
  padding-left: 1.5rem;
  list-style-type: decimal;
}

:deep(.ProseMirror .yRemoteSelection) {
  position: absolute;
  border-left: 2px solid;
  border-right: 2px solid;
  pointer-events: none;
  opacity: 0.5;
}

:deep(.ProseMirror .yRemoteSelectionHead) {
  position: absolute;
  height: 1.2em;
  width: 2px;
  pointer-events: none;
}
</style> 