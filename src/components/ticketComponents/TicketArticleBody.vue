<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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
  content?: string;
}

const props = withDefaults(defineProps<Props>(), {
  content: '',
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

// Create Markdown serializer and parser
const serializer = new MarkdownSerializer(
  defaultMarkdownSerializer.nodes,
  defaultMarkdownSerializer.marks
);
const parser = new MarkdownParser(mySchema, defaultMarkdownParser.tokenizer, defaultMarkdownParser.tokens);

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
const showTypeDropdown = ref(false);
const showInsertDropdown = ref(false);
const showMoreDropdown = ref(false);

// Initialize editor with custom menubar
onMounted(() => {
  if (!editorRef.value) return;

  const plugins = exampleSetup({ schema: mySchema, menuBar: false });

  const state = EditorState.create({
    doc: DOMParser.fromSchema(mySchema).parse(document.createElement('div')),
    plugins: [...plugins, markdownInputRules],
  });

  view = new EditorView(editorRef.value, {
    state,
    dispatchTransaction(transaction) {
      if (!view) return;
      const newState = view.state.apply(transaction);
      view.updateState(newState);
      const markdown = serializer.serialize(newState.doc);
      emit('update:content', markdown);
    },
  });

  // Set initial content
  if (props.content) {
    const doc = parser.parse(props.content);
    if (doc) {
      view.updateState(EditorState.create({
        doc,
        plugins: view.state.plugins,
      }));
    }
  }

  // Debug: Verify marks are available
  console.log('Available marks:', Object.keys(mySchema.marks));
});

onUnmounted(() => {
  if (view) view.destroy();
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
</script>

<template>
  <div class="bg-slate-800 rounded-2xl p-4 shadow-lg">
    <h2 class="text-lg font-medium text-slate-100 mb-3">Ticket Description</h2>
    <div class="editor-wrapper">
      <div class="bg-slate-700 p-2 rounded-t-xl border-b border-slate-600 flex items-center gap-2">
        <!-- Type Dropdown -->
        <div class="relative">
          <button
            @click="showTypeDropdown = !showTypeDropdown"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none"
          >
            Type...
          </button>
          <div v-if="showTypeDropdown" class="absolute top-full left-0 mt-1 bg-slate-700 rounded-md shadow-lg z-10">
            <button @click="setBlockType('paragraph'); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Plain
            </button>
            <button @click="setBlockType('heading', { level: 1 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 1
            </button>
            <button @click="setBlockType('heading', { level: 2 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 2
            </button>
            <button @click="setBlockType('heading', { level: 3 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 3
            </button>
            <button @click="setBlockType('heading', { level: 4 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 4
            </button>
            <button @click="setBlockType('heading', { level: 5 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 5
            </button>
            <button @click="setBlockType('heading', { level: 6 }); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Heading Level 6
            </button>
            <button @click="setBlockType('code_block'); showTypeDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Code
            </button>
          </div>
        </div>

        <!-- Insert Dropdown -->
        <div class="relative">
          <button
            @click="showInsertDropdown = !showInsertDropdown"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none"
          >
            Insert
          </button>
          <div v-if="showInsertDropdown" class="absolute top-full left-0 mt-1 bg-slate-700 rounded-md shadow-lg z-10">
            <button @click="toggleBulletList(); showInsertDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Bullet List
            </button>
            <button @click="toggleOrderedList(); showInsertDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Numbered List
            </button>
            <button @click="insertLink(); showInsertDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Link
            </button>
          </div>
        </div>

        <!-- Formatting Buttons -->
        <button @click="toggleStrong()" class="bg-slate-600 text-slate-200 px-2 py-1 rounded-md hover:bg-slate-500">
          B
        </button>
        <button @click="toggleEm()" class="bg-slate-600 text-slate-200 px-2 py-1 rounded-md hover:bg-slate-500">
          I
        </button>

        <!-- Undo/Redo Buttons -->
        <button @click="undo()" class="bg-slate-600 text-slate-200 px-2 py-1 rounded-md hover:bg-slate-500">
          Undo
        </button>
        <button @click="redo()" class="bg-slate-600 text-slate-200 px-2 py-1 rounded-md hover:bg-slate-500">
          Redo
        </button>

        <!-- More Dropdown -->
        <div class="relative">
          <button
            @click="showMoreDropdown = !showMoreDropdown"
            class="bg-slate-600 text-slate-200 px-3 py-1 rounded-md hover:bg-slate-500 focus:outline-none"
          >
            More
          </button>
          <div v-if="showMoreDropdown" class="absolute top-full left-0 mt-1 bg-slate-700 rounded-md shadow-lg z-10">
            <button @click="liftBlock(); showMoreDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Lift Block
            </button>
            <button @click="selectParent(); showMoreDropdown = false" class="block w-full text-left px-3 py-1 text-slate-200 hover:bg-slate-600">
              Select Parent
            </button>
          </div>
        </div>
      </div>

      <!-- Editor content with click handler -->
      <div
        ref="editorRef"
        @click="focusEditor"
        class="w-full min-h-[400px] p-3 text-slate-200 bg-slate-700 rounded-b-lg border border-slate-600 prose prose-invert max-w-none editor-container"
      ></div>
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
  margin-top: 1rem;
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