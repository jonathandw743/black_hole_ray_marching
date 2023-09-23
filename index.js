const express = require('express');
const app = express();

const path = require('path');

// Serve .wasm files with the correct MIME type
app.get('*.wasm', (req, res, next) => {
  res.set('Content-Type', 'application/wasm');
  next();
});

// app.get('*.html', (req, res, next) => {
//   res.set('Content-Type', 'text/html');
//   next();
// });

// Other routes and middleware

// sendFile will go here
app.get('/', function(req, res) {
    res.sendFile(path.join(__dirname, 'index.html'));
  });
  

// Start the server
app.listen(3000, () => {
  console.log('Server is running on port 3000');
});
