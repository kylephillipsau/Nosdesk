// Simple WebSocket test script

const wsUrl = 'ws://localhost:8080/api/collaboration/ws?docId=test-doc&type=ticket';
console.log(`Attempting to connect to: ${wsUrl}`);

const ws = new WebSocket(wsUrl);

ws.onopen = function() {
  console.log('WebSocket connection established');
  // Send a simple message
  ws.send(JSON.stringify({ type: 'ping' }));
};

ws.onmessage = function(event) {
  console.log('Message received:', event.data);
};

ws.onerror = function(error) {
  console.error('WebSocket error:', error);
};

ws.onclose = function(event) {
  console.log('WebSocket connection closed. Code:', event.code, 'Reason:', event.reason);
};

// Keep the script running for 5 seconds
setTimeout(() => {
  console.log('Closing connection...');
  ws.close();
  process.exit(0);
}, 5000); 