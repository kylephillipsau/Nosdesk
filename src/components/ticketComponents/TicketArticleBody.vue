<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { EditorState, Transaction } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { Schema, DOMParser } from 'prosemirror-model';
import { schema as basicSchema } from 'prosemirror-schema-basic';
import { addListNodes, wrapInList } from 'prosemirror-schema-list';
import { exampleSetup } from 'prosemirror-example-setup';
import { InputRule, inputRules } from 'prosemirror-inputrules';
import { MarkdownSerializer, defaultMarkdownSerializer, MarkdownParser, defaultMarkdownParser } from 'prosemirror-markdown';
import { toggleMark, lift as pmLift, selectParentNode as pmSelectParentNode } from 'prosemirror-commands';
import { undo as pmUndo, redo as pmRedo } from 'prosemirror-history';
import 'prosemirror-view/style/prosemirror.css';

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

// Editor references
const editorRef = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;

// Custom schema with headings, lists, and code blocks
const mySchema = new Schema({
  nodes: addListNodes(basicSchema.spec.nodes.append({
    heading: {
      content: 'inline*',
      group: 'block',
      attrs: { level: { default: 1 } },
      parseDOM: [
        { tag: 'h1', attrs: { level: 1 } },
        { tag: 'h2', attrs: { level: 2 } },
        { tag: 'h3', attrs: { level: 3 } },
        { tag: 'h4', attrs: { level: 4 } },
        { tag: 'h5', attrs: { level: 5 } },
        { tag: 'h6', attrs: { level: 6 } },
      ],
      toDOM(node) { return ['h' + node.attrs.level, 0]; }
    },
    code_block: {
      content: 'text*',
      group: 'block',
      parseDOM: [{ tag: 'pre' }],
      toDOM() { return ['pre', 0]; }
    }
  }), 'paragraph block*', 'block'),
  marks: basicSchema.spec.marks, // Ensure strong and em are included
});

// Create Markdown parser and serializer
const parser = new MarkdownParser(mySchema, defaultMarkdownParser.tokenizer, defaultMarkdownParser.tokens);
const serializer = defaultMarkdownSerializer;

// Define input rules for Markdown-like list behavior
const markdownInputRules = inputRules({
  rules: [
    new InputRule(/^\s*[-*]\s$/, (state: EditorState, match: RegExpMatchArray, start: number, end: number) => {
      const tr = state.tr.delete(start, end);
      return wrapInList(mySchema.nodes.bullet_list)(state) ? tr : null;
    }),
    new InputRule(/^\s*\d+\.\s$/, (state: EditorState, match: RegExpMatchArray, start: number, end: number) => {
      const tr = state.tr.delete(start, end);
      return wrapInList(mySchema.nodes.ordered_list)(state) ? tr : null;
    }),
  ],
});

// Custom dropdown state
const typeMenuRef = ref<HTMLElement | null>(null);
const typeButtonRef = ref<HTMLElement | null>(null);
const insertMenuRef = ref<HTMLElement | null>(null);
const insertButtonRef = ref<HTMLElement | null>(null);
const moreMenuRef = ref<HTMLElement | null>(null);
const moreButtonRef = ref<HTMLElement | null>(null);

const showTypeMenu = ref(false);
const showInsertMenu = ref(false);
const showMoreMenu = ref(false);

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Node;
  
  // Handle Type menu
  if (showTypeMenu.value && typeMenuRef.value && typeButtonRef.value) {
    if (!typeMenuRef.value.contains(target) && !typeButtonRef.value.contains(target)) {
      showTypeMenu.value = false;
    }
  }
  
  // Handle Insert menu
  if (showInsertMenu.value && insertMenuRef.value && insertButtonRef.value) {
    if (!insertMenuRef.value.contains(target) && !insertButtonRef.value.contains(target)) {
      showInsertMenu.value = false;
    }
  }
  
  // Handle More menu
  if (showMoreMenu.value && moreMenuRef.value && moreButtonRef.value) {
    if (!moreMenuRef.value.contains(target) && !moreButtonRef.value.contains(target)) {
      showMoreMenu.value = false;
    }
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    if (showTypeMenu.value) {
      showTypeMenu.value = false;
      typeButtonRef.value?.focus();
    }
    if (showInsertMenu.value) {
      showInsertMenu.value = false;
      insertButtonRef.value?.focus();
    }
    if (showMoreMenu.value) {
      showMoreMenu.value = false;
      moreButtonRef.value?.focus();
    }
  }
};

const toggleTypeMenu = () => {
  showTypeMenu.value = !showTypeMenu.value;
  if (showTypeMenu.value) {
    showInsertMenu.value = false;
    showMoreMenu.value = false;
    nextTick(() => {
      const firstFocusable = typeMenuRef.value?.querySelector('button') as HTMLElement;
      firstFocusable?.focus();
    });
  }
};

const toggleInsertMenu = () => {
  showInsertMenu.value = !showInsertMenu.value;
  if (showInsertMenu.value) {
    showTypeMenu.value = false;
    showMoreMenu.value = false;
    nextTick(() => {
      const firstFocusable = insertMenuRef.value?.querySelector('button') as HTMLElement;
      firstFocusable?.focus();
    });
  }
};

const toggleMoreMenu = () => {
  showMoreMenu.value = !showMoreMenu.value;
  if (showMoreMenu.value) {
    showTypeMenu.value = false;
    showInsertMenu.value = false;
    nextTick(() => {
      const firstFocusable = moreMenuRef.value?.querySelector('button') as HTMLElement;
      firstFocusable?.focus();
    });
  }
};

// Event listeners for click outside
onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});

// Function to focus the editor
const focusEditor = () => {
  if (view) {
    view.focus();
  }
};

// Functions to handle actions
const setBlockType = (type: string, attrs: any = {}) => {
  if (!view) return;
  const { state, dispatch } = view;
  const { tr } = state;
  const { from, to } = state.selection;
  tr.setBlockType(from, to, mySchema.nodes[type], attrs);
  dispatch(tr);
};

const toggleBulletList = () => {
  if (!view) return;
  wrapInList(mySchema.nodes.bullet_list)(view.state, view.dispatch);
};

const toggleOrderedList = () => {
  if (!view) return;
  wrapInList(mySchema.nodes.ordered_list)(view.state, view.dispatch);
};

const insertLink = () => {
  if (!view) return;
  const { state, dispatch } = view;
  const url = prompt('Enter URL for the link:', 'https://');
  if (url) {
    const { from, to } = state.selection;
    const tr = state.tr;
    if (from === to) {
      const text = prompt('Enter link text:', 'Link');
      if (text) {
        tr.insertText(text, from, from);
        tr.addMark(from, from + text.length, mySchema.marks.link.create({ href: url }));
      }
    } else {
      tr.addMark(from, to, mySchema.marks.link.create({ href: url }));
    }
    dispatch(tr);
  }
};

const toggleStrong = () => {
  if (!view) return;
  const command = toggleMark(mySchema.marks.strong);
  const canApply = command(view.state);
  console.log('Can apply strong:', canApply); // Debug
  if (canApply) {
    command(view.state, view.dispatch);
  }
};

const toggleEm = () => {
  if (!view) return;
  const command = toggleMark(mySchema.marks.em);
  const canApply = command(view.state);
  console.log('Can apply em:', canApply); // Debug
  if (canApply) {
    command(view.state, view.dispatch);
  }
};

const undo = () => {
  if (!view) return;
  pmUndo(view.state, view.dispatch);
};

const redo = () => {
  if (!view) return;
  pmRedo(view.state, view.dispatch);
};

const liftBlock = () => {
  if (!view) return;
  pmLift(view.state, view.dispatch);
};

const selectParent = () => {
  if (!view) return;
  pmSelectParentNode(view.state, view.dispatch);
};

onMounted(() => {
  if (!editorRef.value) return;

  const state = EditorState.create({
    schema: mySchema,
    plugins: [...exampleSetup({ schema: mySchema, menuBar: false }), markdownInputRules],
    doc: props.initialContent ? parser.parse(props.initialContent) : undefined
  });

  view = new EditorView(editorRef.value, {
    state,
    dispatchTransaction(transaction: Transaction) {
      if (!view) return;
      
      view.updateState(view.state.apply(transaction));
      
      // Only emit content changes if the document actually changed
      if (transaction.docChanged) {
        const content = serializer.serialize(view.state.doc);
        emit('update:content', content);
      }
    }
  });
});
</script>

<template>
  <div class="bg-slate-800 rounded-2xl p-2 shadow-lg">
    <div class="text-lg font-medium text-slate-100 p-4 py-2">Ticket Notes</div>
    <div class="editor-wrapper">
      <div class="bg-slate-700 p-2 rounded-lg border-slate-800 flex items-center gap-2">
        <!-- Type Dropdown -->
        <div class="relative">
          <button
            ref="typeButtonRef"
            @click="toggleTypeMenu"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            aria-haspopup="true"
            :aria-expanded="showTypeMenu"
          >
            Type
          </button>

          <!-- Type Menu Dropdown -->
          <div
            v-if="showTypeMenu"
            ref="typeMenuRef"
            class="absolute top-full left-0 mt-1 w-48 bg-slate-700 rounded-md shadow-lg py-1 z-10"
            role="menu"
            tabindex="-1"
          >
            <button
              v-for="(type, index) in [
                { label: 'Plain', action: () => setBlockType('paragraph') },
                { label: 'Heading Level 1', action: () => setBlockType('heading', { level: 1 }) },
                { label: 'Heading Level 2', action: () => setBlockType('heading', { level: 2 }) },
                { label: 'Heading Level 3', action: () => setBlockType('heading', { level: 3 }) },
                { label: 'Heading Level 4', action: () => setBlockType('heading', { level: 4 }) },
                { label: 'Heading Level 5', action: () => setBlockType('heading', { level: 5 }) },
                { label: 'Heading Level 6', action: () => setBlockType('heading', { level: 6 }) },
                { label: 'Code', action: () => setBlockType('code_block') }
              ]"
              :key="index"
              @click="type.action(); showTypeMenu = false"
              class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600 focus:bg-slate-600 focus:outline-none"
              role="menuitem"
            >
              {{ type.label }}
            </button>
          </div>
        </div>

        <!-- Insert Dropdown -->
        <div class="relative">
          <button
            ref="insertButtonRef"
            @click="toggleInsertMenu"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            aria-haspopup="true"
            :aria-expanded="showInsertMenu"
          >
            Insert
          </button>

          <!-- Insert Menu Dropdown -->
          <div
            v-if="showInsertMenu"
            ref="insertMenuRef"
            class="absolute top-full left-0 mt-1 w-48 bg-slate-700 rounded-md shadow-lg py-1 z-10"
            role="menu"
            tabindex="-1"
          >
            <button
              v-for="(item, index) in [
                { label: 'Bullet List', action: toggleBulletList },
                { label: 'Numbered List', action: toggleOrderedList },
                { label: 'Link', action: insertLink }
              ]"
              :key="index"
              @click="item.action(); showInsertMenu = false"
              class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600 focus:bg-slate-600 focus:outline-none"
              role="menuitem"
            >
              {{ item.label }}
            </button>
          </div>
        </div>

        <!-- Formatting Buttons -->
        <button
          @click="toggleStrong"
          class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Bold
        </button>
        <button
          @click="toggleEm"
          class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Italic
        </button>

        <!-- More Dropdown -->
        <div class="relative ml-auto">
          <button
            ref="moreButtonRef"
            @click="toggleMoreMenu"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
            aria-haspopup="true"
            :aria-expanded="showMoreMenu"
          >
            More
          </button>

          <!-- More Menu Dropdown -->
          <div
            v-if="showMoreMenu"
            ref="moreMenuRef"
            class="absolute top-full right-0 mt-1 w-48 bg-slate-700 rounded-md shadow-lg py-1 z-10"
            role="menu"
            tabindex="-1"
          >
            <button
              v-for="(item, index) in [
                { label: 'Lift Block', action: liftBlock },
                { label: 'Select Parent', action: selectParent }
              ]"
              :key="index"
              @click="item.action(); showMoreMenu = false"
              class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600 focus:bg-slate-600 focus:outline-none"
              role="menuitem"
            >
              {{ item.label }}
            </button>
          </div>
        </div>
      </div>

      <!-- Editor content with click handler -->
      <div ref="editorRef" @click="focusEditor"
        class="w-full min-h-[400px] p-3 text-slate-200 bg-slate-800 rounded-b-lg prose prose-invert max-w-none editor-container">
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-wrapper {
  position: relative;
}

/* Menubar styling */
.menubar {
  background-color: #374151;
  padding: 0.5rem;
  border-radius: 0.5rem 0.5rem 0 0;
  display: flex;
  gap: 0.25rem;
  align-items: center;
  border-bottom: 1px solid #4a5568;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.menubar button {
  background-color: #4a5568;
  color: #e5e7eb;
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  border: none;
  cursor: pointer;
  font-size: 0.75rem;
  height: 28px;
  display: flex;
  align-items: center;
  line-height: 1;
  transition: background-color 0.2s ease;
}

.menubar button:hover {
  background-color: #6b7280;
}

.menubar button:active {
  background-color: #4a5568;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  background-color: #374151;
  border-radius: 0.375rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  min-width: 120px;
  z-index: 10;
}

.dropdown-menu button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 0.25rem 0.5rem;
  background-color: transparent;
  color: #e5e7eb;
  border: none;
  cursor: pointer;
  font-size: 0.75rem;
}

.dropdown-menu button:hover {
  background-color: #4a5568;
}

:deep(.ProseMirror) {
  border: none !important;
  outline: none !important;
}

:deep(.ProseMirror-focused) {
  border: none !important;
  outline: none !important;
}

/* Ensure bold renders correctly */
:deep(strong) {
  font-weight: bold !important;
}

/* Ensure italics renders correctly */
:deep(em) {
  font-style: italic !important;
}

.editor-container {
  cursor: text;
}

:deep(h1), :deep(h2), :deep(h3), :deep(h4), :deep(h5), :deep(h6) {
    margin-top: 0.2rem;
    margin-bottom: 0.5rem;
  }

:deep(h1) { font-size: 1.875rem; font-weight: bold; }
:deep(h2) { font-size: 1.5rem; font-weight: bold; }
:deep(h3) { font-size: 1.25rem; font-weight: bold; }
:deep(h4) { font-size: 1.125rem; font-weight: bold; }
:deep(h5) { font-size: 1rem; font-weight: bold; }
:deep(h6) { font-size: 0.875rem; font-weight: bold; }

:deep(pre) {
  background-color: #2d2d2d;
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
}

:deep(code) {
  background-color: #2d2d2d;
  padding: 0.2rem 0.4rem;
  border-radius: 0.25rem;
}

:deep(a) {
  color: #60a5fa;
  text-decoration: underline;
}

:deep(ul) {
  list-style-type: disc;
  padding-left: 1.5rem;
}

:deep(ol) {
  list-style-type: decimal;
  padding-left: 1.5rem;
}

:deep(li) {
  margin-bottom: 0.25rem;
}
</style>