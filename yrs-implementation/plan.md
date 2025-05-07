# Yrs implementation
## Backend implementation
- The backend handles the Yjs document state with the yrs crate, the rust implementation of yjs
- The backend keeps a cached version of the Yjs document while it is being worked on while there is an established WebSocket connection
- Yjs clients in the frontend connect to the Yrs WebSocket provider handled by the backend
- When the clients disconnect, the Yjs binary is then converted to a serialised JSON document and then saved to the database as a version of the document to support version history
- Ticket Articles and Documentation Pages share an almost identical structure, but are just separated into two different sections for organisational purposes
- The Yjs awareness protocol is supported, allowing users to see real time updates from other clients, parsing the associated user information

### Improved Backend Implementation (Best Practices)

#### Document State Management
- Use `Doc` as the core structure with properly generated `client_id` to ensure uniqueness across peers
- Store document state in binary format using `Update::encode_v1` or `Update::encode_v2` for efficiency
- Consider storing documents directly in binary format rather than JSON for better performance and data integrity
- Implement both state vector synchronization and update-based propagation for reliable sync

#### Document Lifecycle
1. **Initialization Phase**:
   - When a WebSocket connection is established, load the document from the database
   - Decode stored binary updates using `Update::decode_v1/v2` and apply to the in-memory `Doc`
   - Generate an initial state vector to be sent to the client using `txn.state_vector().encode_v1()`

2. **Active Editing Phase**:
   - Handle incoming WebSocket messages according to the Y-protocol:
     - Message type 0: Sync protocol (applying updates)
     - Message type 1: Awareness protocol (cursor position, user info)
   - Apply incoming updates to the document using `txn.apply_update(Update::decode_v1(...))`
   - Broadcast updates to all connected clients except the sender

3. **Saving Phase**:
   - Implement periodic saving (every 30-60 seconds) instead of only on disconnect
   - Use a throttled save mechanism to prevent excessive database writes
   - When saving, acquire a read transaction and encode the full document state: `txn.encode_state_as_update_v1(&StateVector::default())`
   - Maintain a version history by storing timestamped snapshots

#### Error Handling & Recovery
- Implement proper connection cleanup to prevent memory leaks
- Handle WebSocket disconnections gracefully with automatic reconnection on the client
- Use proper error handling with the `?` operator and meaningful error messages
- Implement document state verification to detect corruption

## Frontend implementation
- The frontend uses Yjs for CRDT handling and ProseMirror for the markdown editor. The client process is only responsible for supporting a WebSocket connection and providing the markdown editor frontend environment.

### Improved Frontend Implementation (Best Practices)

#### Document Synchronization
- Use `WebsocketProvider` from the y-websocket module for connection management
- Implement proper connection status indicators (connected, disconnecting, offline)
- Use the awareness protocol to show collaborative cursors and user presence
- Handle reconnection automatically with exponential backoff

#### Editor Integration
- Connect the Yjs document to ProseMirror using the y-prosemirror bindings
- Utilize `ySyncPlugin`, `yCursorPlugin`, and `yUndoPlugin` for complete integration
- Support formatting and embedded content properly as described in the Yrs documentation
- Implement proper cursor positioning using StickyIndex for reliable cursor placement

#### Offline Support
- Implement local persistence using IndexedDB (y-indexeddb provider)
- Store updates locally when offline and sync when connection is restored
- Show clear indicators when working in offline mode

#### Performance Considerations
- Use efficient update encoding (v2 when available)
- Batch updates when possible to reduce network traffic
- Consider implementing a "quieter" mode during high traffic periods

## Sync Protocol Implementation

The sync between clients follows this pattern:
1. **Initial sync**:
   - Client connects and sends its state vector
   - Server responds with updates the client hasn't seen yet
   - Client applies these updates and is now in sync

2. **Ongoing sync**:
   - Client makes changes locally which generate updates
   - Updates are sent to the server via WebSocket
   - Server applies updates to its document
   - Server broadcasts updates to all other connected clients
   - Other clients apply these updates

3. **Reconnection sync**:
   - When a client reconnects, it sends its current state vector
   - Server computes the difference using `txn.encode_diff_v1(sv)`
   - Client receives and applies missing updates

The protocol should handle merge conflicts automatically through the CRDT algorithm, with no additional conflict resolution needed.

## Document Storage Strategy

1. **Storage Format**:
   - Store the document as binary Yjs updates using `encode_state_as_update_v1/v2`
   - This is more space-efficient than JSON and preserves all CRDT metadata

2. **Versioning Strategy**:
   - Store periodic snapshots (every 50 updates or 5 minutes)
   - Each snapshot contains the full document state at that point
   - Implement the Snapshot feature from Yrs to access past document states

3. **Performance Optimization**:
   - Cache frequently accessed documents in memory
   - Implement document garbage collection for unused documents
   - Use the DeleteSet and update merging to keep document size manageable